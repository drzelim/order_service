use axum::{
    extract::{Path, State}, http::StatusCode, Json, response::{IntoResponse, Response}
};
use std::{env, sync::Arc};
use tokio::time::{ timeout, Duration};
use serde_json::Value;
use log::{info, error};
use tokio::sync::Mutex as TokioMutex;
use tokio_postgres::Client;
use lru::LruCache;
use std::num::NonZeroUsize;
use lazy_static::lazy_static;

use crate::{db, models::Order};

lazy_static! {
    pub static ref REQUEST_DURATION: u64 = { 
        env::var("REQUEST_DURATION").expect("REQUEST_DURATION must be set").parse::<u64>().unwrap()
    }; 
}

pub const DATA_KEY: &str = "data";

pub struct AppState {
    pub db_client: Arc<TokioMutex<Client>>,
    pub cache: Arc<TokioMutex<LruCache<String, Order>>>,
}

impl AppState {
    pub fn new(db_client: Client, max_cache_size: NonZeroUsize) -> Self {
        AppState {
            db_client: Arc::new(TokioMutex::new(db_client)),
            cache: Arc::new(TokioMutex::new(LruCache::new(max_cache_size))),
        }
    }

    pub async fn insert(&self, key: &str, value: Order) {
        let mut cache = self.cache.lock().await;
        cache.put(key.to_string(), value);
    }

    pub async fn get(&self, key: &str) -> Option<Order> {
        let mut cache = self.cache.lock().await;
        cache.get(key).cloned()
    }
}

#[derive(Debug)]
pub struct CustomResponse {
    pub message: String,
    pub code: StatusCode,
}

impl IntoResponse for CustomResponse {
    fn into_response(self) -> Response {
        (self.code, self.message).into_response()
    }
}

pub async fn get_order_by_uid(
    state: State<Arc<AppState>>,
    Path(order_uid): Path<String>,
) -> Result<Json<Option<Order>>, CustomResponse> {

    if let Some(cached_order) = state.get(&order_uid).await {
        info!("Order {} found in cache", &order_uid);
        return Ok(Json(Some(cached_order)));
    }

    let duration = Duration::from_secs(*REQUEST_DURATION);
    let result = timeout(duration, db::get_order_by_uid(&state, &order_uid)).await;

    // Обработка таймута
    let result = match result {
        Ok(Ok(Some(row))) => {
            // Извлекаем JSON из строки данных
            let data: Value = row.get("data");
            let order: Order = serde_json::from_value(data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).unwrap();

             // Успешный запрос и десериализация
            Ok(Json(Some(order)))
        }

        Ok(Ok(None)) => Err(CustomResponse {
            message: "Not fount".into(),
            code: StatusCode::NOT_FOUND
        }),

        Ok(Err(_)) => Err(CustomResponse {
            message: "Server error".into(),
            code: StatusCode::INTERNAL_SERVER_ERROR
        }),

        Err(_) => Err(CustomResponse {
            message: "Database operation timed out".into(),
            code: StatusCode::REQUEST_TIMEOUT
        })
    };

    match &result {
        Ok(Json(Some(order))) => {
            // Добавляем в кэш
            state.insert(&order.order_uid, order.clone()).await;
        },
        Ok(Json(None)) => {},
        Err(res) => error!("Failed to retrieve order: {}", res.code),
    };

    result
}

// Для проверки соответствия получаемых данных используется встроенный в Axum механизм Json<Order>
// Это гарантирует, что данные будут соответствовать структуре Order.
pub async fn add_order(
    State(state): State<Arc<AppState>>,
    Json(order): Json<Order>,
) -> CustomResponse {

    state.insert(&order.order_uid, order.clone()).await;

    let duration = Duration::from_secs(*REQUEST_DURATION);
    let result = timeout(duration, db::insert_order(&state, &order, DATA_KEY)).await;

    match result {
        Ok(Ok(_)) => {
            CustomResponse {
                message: "Order successfully created".into(),
                code: StatusCode::CREATED
            }
        },
        Ok(Err(ref e)) if db::is_unique_violation(e) => {
            CustomResponse {
                message: "Order already exists".into(),
                code: StatusCode::CONFLICT,
            }
        }
        Ok(Err(_)) => {
            CustomResponse {
                message: "Error".into(),
                code: StatusCode::INTERNAL_SERVER_ERROR
            } 
        },
        Err(_) => {
            CustomResponse {
                message: "Database operation timed out".into(),
                code: StatusCode::REQUEST_TIMEOUT
            }
        }
    }
}
