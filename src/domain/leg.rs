use crate::Greeks;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leg {
    pub symbol: String,
    pub strike: Option<Decimal>,   // none for stock
    pub quantity: i32,             // negative = short
    pub price_paid: Decimal,       // +collected, –paid
    pub expiry: Option<NaiveDate>, // none for stock
    pub greeks: Greeks,
}

impl Leg {
    pub fn market_value(&self) -> Decimal {
        (self.price_paid * Decimal::from_i32(self.quantity).unwrap())
    }
}
