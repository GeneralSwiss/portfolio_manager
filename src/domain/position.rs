use crate::PositionType;
use crate::domain::leg::Leg;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub pos_type: PositionType,
    pub legs: Vec<Leg>,
    pub margin_used: Decimal,
}

impl Position {}
