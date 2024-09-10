use lazy_static::lazy_static;
use std::env;
use std::sync::Arc;
use tokio_postgres::{Client, Error, NoTls, Row};
use std::option::Option;
use log:: error;

use crate::handlers::AppState;
use crate::models::Order;

lazy_static! {
    static ref DATABASE_TABLE: String = { 
        env::var("DATABASE_TABLE").expect("DATABASE_TABLE must be set") 
    };
}

pub async fn connect() -> Result<Client, Error> {
    let database_user = env::var("DATABASE_USER").expect("DATABASE_USER must be set");
    let database_password = env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD must be set");

    let database_url = format!("host=localhost user={database_user} password={database_password}");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
            
        }
    });

    /*
       Для удобства, принято решение хранить заказы в формате jsonb.
       Однако, если важна производительность, возможно лучше будет использовать
       классический реляционный подход с созданием таблиц под каждую сущность.
    */

    // Запрос на создание таблицы, если еще не создана
    let query = format!(
        "
        CREATE TABLE IF NOT EXISTS {} (
            order_uid TEXT PRIMARY KEY,
            data JSONB NOT NULL
        );
    ",
        *DATABASE_TABLE
    );

    client.execute(&query, &[]).await?;

    Ok(client)
}

// Добавление заказа в БД
pub async fn insert_order(state: &Arc<AppState>, order: &Order, key: &str) -> Result<u64, Error> {
    let order_json = serde_json::to_value(order).unwrap();

    // Используем транзакции для сохрарения данных в случае ошибки
    let mut client = state.db_client.lock().await;
    let transaction = client.transaction().await?;

    let result = transaction
        .execute(
            &format!(
                "INSERT INTO {} (order_uid, {}) VALUES ($1, $2)",
                *DATABASE_TABLE, key
            ),
            &[&order.order_uid, &order_json],
        ).await;

    match result {
        Ok(row) => {
            transaction.commit().await?;
            Ok(row)
        }
        Err(err) => {
            transaction.rollback().await?;
            Err(err)
        }
    }
}

// Поиск заказа в БД
pub async fn get_order_by_uid(
    state: &Arc<AppState>,
    order_uid: &str,
) -> Result<Option<Row>, Error> {

    let mut client = state.db_client.lock().await;
    let transaction = client.transaction().await?;

    let result = transaction.query_opt(
        &format!("SELECT data FROM {} WHERE order_uid = $1", *DATABASE_TABLE),
        &[&order_uid],
    ).await;

    match result {
        Ok(row) => {
            transaction.commit().await?;
            Ok(row)
        }
        Err(err) => {
            transaction.rollback().await?;
            Err(err)
        }
    }
}

pub fn is_unique_violation(error: &tokio_postgres::Error) -> bool {
    if let Some(db_error) = error.as_db_error() {
        db_error.code() == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION
    } else {
        false
    }
}
