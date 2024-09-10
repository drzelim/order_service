use axum::{
    routing::{get, post}, Router
};
use dotenvy::dotenv;
use std::{env, sync::Arc};
use std::num::NonZeroUsize;
use clap::Parser;
use lazy_static::lazy_static;
use log::info;
use env_logger;

mod db;
mod models;
mod handlers;

use handlers::{add_order, get_order_by_uid, AppState};

lazy_static! {
    pub static ref PORT: String = { 
        env::var("PORT").expect("PORT must be set") 
    };
    
    pub static ref CACHE_SIZE: usize = { 
        env::var("CACHE_SIZE").expect("CACHE_SIZE must be set").parse::<usize>().unwrap()
    };
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = PORT.to_string())]
    port: String,

    #[arg(short, long, default_value_t = *CACHE_SIZE)]
    cachesize: usize
}

#[tokio::main]
pub async fn main() {
    // Загрузка переменных окружения
    dotenv().ok();

    // Инициализация логирования
    env_logger::init();

    // Инициализация аргументов командной строки
    let args = Args::parse();

    // Подключение к базе данных
    let client = db::connect().await.expect("Database connection error");

    // Создание состояния приложения
    let app_state = Arc::new(AppState::new(client, NonZeroUsize::new(args.cachesize).unwrap()));

    // Создание маршрутов
    let app = Router::new()
        .route("/orders/:order_uid", get(get_order_by_uid))
        .route("/order", post(add_order)) 
        .with_state(app_state);

    // Запуск сервера
    let addr: &str = &format!("0.0.0.0:{}", args.port);
    info!("Starting server at {}", &addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
