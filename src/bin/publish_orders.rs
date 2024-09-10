use dotenvy::dotenv;
use reqwest::Client;
use core::str;
use std::{env, error::Error};
use lazy_static::lazy_static;
use log::{info, error};
use env_logger;

use order_service::{helpers::get_mock_order, models};
use models::Order;


lazy_static! {
    static ref PORT: String = { 
        env::var("PORT").expect("PORT must be set") 
    };
}

async fn publish_order(api_url: &str, order: &Order) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .post(api_url)
        .json(&order)
        .send()
        .await?;

    if response.status().is_success() {
        info!("Order published successfully: {:?}", order);
    } else {
        error!("Failed to publish order. Status: {}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let api_url = &format!("http://localhost:{}/order", *PORT);

    for item in 1..11 {
        let test_order = get_mock_order(item.to_string());
        publish_order(api_url, &test_order).await?;
    }

    Ok(())
}