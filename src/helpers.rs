use rand::Rng;
use rand::distributions::Alphanumeric;
use chrono::Utc;

use crate::models;
use models::{Order, Payment, Delivery, Item};


pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn get_mock_order(id: String) -> Order {
    let mut rng = rand::thread_rng();

    // Генерация случайных данных для заказа
    let order_uid = id;
    let track_number = generate_random_string(12);
    let entry = "WBIL".to_string();
    let delivery = Delivery {
        name: "Test Testov".to_string(),
        phone: format!("+972{}", rng.gen_range(10000000..99999999)),
        zip: format!("{}", rng.gen_range(1000000..9999999)),
        city: "Kiryat Mozkin".to_string(),
        address: "Ploshad Mira 15".to_string(),
        region: "Kraiot".to_string(),
        email: format!("test{}@gmail.com", rng.gen_range(1..100)),
    };

    let payment = Payment {
        transaction: order_uid.clone(),
        request_id: "".to_string(),
        currency: "USD".to_string(),
        provider: "wbpay".to_string(),
        amount: rng.gen_range(1000..5000),
        payment_dt: Utc::now().timestamp() as u64,
        bank: "alpha".to_string(),
        delivery_cost: rng.gen_range(100..500),
        goods_total: rng.gen_range(500..2000),
        custom_fee: 0,
    };

    let items = vec![
        Item {
            chrt_id: rng.gen_range(1000000..9999999),
            track_number: track_number.clone(),
            price: rng.gen_range(100..500),
            rid: generate_random_string(16),
            name: "Random Item".to_string(),
            sale: rng.gen_range(0..50),
            size: "L".to_string(),
            total_price: rng.gen_range(100..500),
            nm_id: rng.gen_range(1000000..9999999),
            brand: "Random Brand".to_string(),
            status: 202,
        },
    ];

    Order {
        order_uid,
        track_number,
        entry,
        delivery,
        payment,
        items,
        locale: "en".to_string(),
        internal_signature: "".to_string(),
        customer_id: "random_customer".to_string(),
        delivery_service: "meest".to_string(),
        shardkey: rng.gen_range(1..10).to_string(),
        sm_id: rng.gen_range(1..100),
        date_created: Utc::now().to_rfc3339(),
        oof_shard: rng.gen_range(1..10).to_string(),
    }
}
