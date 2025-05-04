use crate::domain::leg::Leg;
use crate::{BookLayer, PositionType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub underlying: String, // "SPY", "QQQ", etc.
    pub book_layer: BookLayer,
    pub pos_type: PositionType,
    pub legs: Vec<Leg>,
    pub margin_used: f64,
}

impl Position {
    pub fn market_value(&self) -> f64 {
        if (self.legs.is_empty()) {
            return 0.0;
        }
        self.legs.iter().map(|l| l.market_value()).sum()
    }
}
