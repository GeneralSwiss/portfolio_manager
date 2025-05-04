use crate::domain::leg::Leg;
use crate::{BookLayer, PositionType};
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub underlying: String, // "SPY", "QQQ", etc.
    pub book_layer: BookLayer,
    pub pos_type: PositionType,
    pub legs: Vec<Leg>,
    pub margin_used: Decimal,
}

impl Position {
    pub fn market_value(&self) -> Decimal {
        if (self.legs.is_empty()) {
            return Decimal::zero();
        }
        self.legs.iter().map(|l| l.market_value()).sum()
    }
}
