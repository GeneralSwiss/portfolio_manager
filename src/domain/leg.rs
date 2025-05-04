use crate::Greeks;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leg {
    pub leg_type: LegType,
    pub strike: f64,
    pub quantity: i32, // negative = short
    pub premium: f64,  // +collected, –paid
    pub expiry: NaiveDate,
    pub greeks: Greeks,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LegType {
    Long,
    Short,
}

impl Leg {
    pub fn market_value(&self) -> f64 {
        self.quantity as f64 * self.premium
    }
}
