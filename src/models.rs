use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Delivery {
    pub name: String,
    pub phone: String,
    pub zip: String,
    pub city: String,
    pub address: String,
    pub region: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
    pub transaction: String,
    pub request_id: String,
    pub currency: String,
    pub provider: String,
    pub amount: u64,
    pub payment_dt: u64,
    pub bank: String,
    pub delivery_cost: u64,
    pub goods_total: u64,
    pub custom_fee: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub chrt_id: u64,
    pub track_number: String,
    pub price: u64,
    pub rid: String,
    pub name: String,
    pub sale: u64,
    pub size: String,
    pub total_price: u64,
    pub nm_id: u64,
    pub brand: String,
    pub status: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub order_uid: String,
    pub track_number: String,
    pub entry: String,
    pub delivery: Delivery,
    pub payment: Payment,
    pub items: Vec<Item>,
    pub locale: String,
    pub internal_signature: String,
    pub customer_id: String,
    pub delivery_service: String,
    pub shardkey: String,
    pub sm_id: u64,
    pub date_created: String,
    pub oof_shard: String,
}
