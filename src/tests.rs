#[cfg(test)]
mod tests {
    use axum::{
        extract::{Path, State}, http::StatusCode, Error, Json
    };
    use lru::LruCache;
    use std::{env, num::NonZeroUsize};
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;
    use dotenvy::dotenv;
    use ctor::ctor;

    use crate::{ handlers::{add_order, get_order_by_uid, DATA_KEY}, models::Order};
    use crate::{
        db,
        handlers::{AppState, CustomResponse},
        helpers::get_mock_order,
    };

    #[ctor]
    fn setup() {
        dotenv().ok();

        let database_table = env::var("DATABASE_TABLE").expect("DATABASE_TABLE must be set");
        env::set_var("DATABASE_TABLE", &format!("test_{database_table}"));
    }

    async fn truncate_test_table(state: &Arc<AppState>, table: String) -> Result<(), Error>  {
        if !table.starts_with("test_") {
            panic!("Attempt to truncate non-test table")
        }
    
        let client = state.db_client.lock().await;

        let query = format!("TRUNCATE TABLE {}", table);
        client.execute(&query, &[]).await.unwrap();
    
        Ok(())
    }

    async fn start_settings(order_uid: String) -> Result<(Arc<AppState>, Order), Error>{
        let test_table = env::var("DATABASE_TABLE").expect("DATABASE_TABLE must be set");

        let cache = LruCache::new(NonZeroUsize::new(10).unwrap());

        let test_order = get_mock_order(order_uid.clone());

        let client = db::connect().await.expect("Database connection error");

        let state = Arc::new(AppState {
            cache: Arc::new(TokioMutex::new(cache)),
            db_client: Arc::new(TokioMutex::new(client)),
        });

        truncate_test_table(&state, test_table).await.expect("Error");

        Ok((state, test_order))
    }


    // Мокирование зависимостей для теста
    #[tokio::test]
    async fn test_get_order_by_uid_from_cache() {
        let order_uid = "test_uid".to_string();
        
        let (state, test_order) = start_settings(order_uid.clone()).await.unwrap();

        // 1. Добавляем заказ в кэш
        state.insert(&order_uid, test_order.clone()).await;

        // 2. Вызов хендлера с переданным `order_uid`
        let result = get_order_by_uid(State(state), Path(order_uid.clone())).await;

        // 3. Проверка результата
        match result {
            Ok(Json(Some(order))) => {
                assert_eq!(order.order_uid, test_order.order_uid);
            }
            _ => panic!("Test failed: expected order from cache"),
        }
    }

    #[tokio::test]
    async fn test_get_order_by_uid_from_db() {
        // Создание тестового состояния с пустым кэшем
        let order_uid = "test_get_order_by_uid_from_db".to_string();
        let (state, test_order) = start_settings(order_uid.clone()).await.unwrap();

        db::insert_order(&state, &test_order, "data").await.unwrap();

        let result = get_order_by_uid(State(state), Path(order_uid.clone())).await;

        match result {
            Ok(Json(Some(order))) => {
                assert_eq!(order.order_uid, test_order.order_uid);
            }
            _ => panic!("Test failed: expected order from DB"),
        }
    }

    #[tokio::test]
    async fn test_get_order_by_uid_not_found() {
        let order_uid = "non_existing_uid".to_string();
        let (state, _) = start_settings(order_uid.clone()).await.unwrap();

        //Вызов хендлера с несуществующим `order_uid`
        let result = get_order_by_uid(State(state), Path(order_uid.clone())).await;

        match result {
            Err(CustomResponse { code, message }) => {
                assert_eq!(code, StatusCode::NOT_FOUND);
                assert_eq!(message, "Not fount");
            }
            _ => panic!("Test failed: expected Not Found error"),
        }
    }

    #[tokio::test]
    async fn test_add_order_success() {
        let order_uid = "test_add_order_success".to_string();
        let (state, test_order) = start_settings(order_uid.clone()).await.unwrap();
    
        // Запуск тестируемой функции
        let response = add_order(
            State(state),
            Json(test_order.clone()),
        ).await;
    
        // Проверка результата
        assert_eq!(response.code, StatusCode::CREATED);
        assert_eq!(response.message, "Order successfully created");
    }

    #[tokio::test]
    async fn test_add_order_conflict() {
        let order_uid = "test_add_order_conflict".to_string();
        let (state, test_order) = start_settings(order_uid.clone()).await.unwrap();

        db::insert_order(&state, &test_order, DATA_KEY).await.unwrap();

        // Запуск тестируемой функции
        let response = add_order(
            State(state),
            Json(test_order.clone()),
        ).await;

        // Проверка результата
        assert_eq!(response.code, StatusCode::CONFLICT);
        assert_eq!(response.message, "Order already exists");
    }

}
