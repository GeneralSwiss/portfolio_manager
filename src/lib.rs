#![allow(dead_code)]
pub mod domain;
mod error;
mod ibkr;
mod repo;
mod ui;

use std::ops::Add;

pub use error::{Error, ExpirationDateParseErrorKind, Result};

use crate::domain::Leg;
pub use crate::domain::Position;
pub use repo::Repo;
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};
pub use ui::main_page;

/// --------------------------- CORE DOMAIN ---------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PositionType {
    CreditSpread,
    DebitSpread,
    LongCall,
    LongPut,
    ShortCall,
    ShortPut,
    Stock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BookLayer {
    Income(Position),    // credit spreads, theta engines
    Insurance(Position), // convex tail-risk hedges
}

impl BookLayer {
    pub fn new_insurance(pos_type: PositionType, legs: Vec<Leg>) -> Self {
        Self::Insurance(Position {
            pos_type,
            legs,
            margin_used: Decimal::zero(),
        })
    }

    pub fn new_income(position_type: PositionType, legs: Vec<Leg>) -> Self {
        Self::Income(Position {
            pos_type: position_type,
            legs,
            margin_used: Decimal::zero(),
        })
    }
}

impl AsRef<Position> for BookLayer {
    fn as_ref(&self) -> &Position {
        match self {
            BookLayer::Income(p) => p,
            BookLayer::Insurance(p) => p,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Greeks {
    pub delta: Decimal,
    pub vega: Option<Decimal>,
    pub theta: Option<Decimal>,
    pub gamma: Option<Decimal>,
    pub rho: Option<Decimal>,
}

impl Greeks {
    pub fn from_stock(quantity: f64) -> Greeks {
        if quantity < 0.0 {
            Greeks {
                delta: Decimal::NEGATIVE_ONE,
                ..Default::default()
            }
        } else {
            Greeks {
                delta: Decimal::ONE,
                ..Default::default()
            }
        }
    }
}

impl Add for Greeks {
    type Output = Greeks;

    fn add(self, rhs: Self) -> Self::Output {
        Greeks {
            delta: self.delta + rhs.delta,
            gamma: self.gamma.zip(rhs.gamma).map(|(lhs, rhs)| lhs + rhs),
            vega: self.vega.zip(rhs.vega).map(|(lhs, rhs)| lhs + rhs),
            theta: self.theta.zip(rhs.theta).map(|(lhs, rhs)| lhs + rhs),
            rho: self.rho.zip(rhs.rho).map(|(lhs, rhs)| lhs + rhs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub cash: Decimal,
    pub positions: Vec<BookLayer>,
}

/// --------------------------- AGGREGATION HELPERS ---------------------------

impl Portfolio {
    /// Net portfolio Greeks (beta-weighted later if desired)
    pub fn net_greeks(&self) -> Greeks {
        self.positions
            .iter()
            .flat_map(|p| &p.as_ref().legs)
            .fold(Default::default(), |acc, leg| acc + leg.greeks())
    }

    /// Total margin in use
    pub fn margin_used(&self) -> Decimal {
        self.positions
            .iter()
            .map(|p| p.as_ref().margin_used)
            .sum::<Decimal>()
    }

    /// Convexity-coverage ratio (stub: replace `shock_payout` with real pricer)
    pub fn convexity_ratio(&self, shock_payout: Decimal) -> Decimal {
        let spread_risk: Decimal = self
            .positions
            .iter()
            .filter_map(|p| {
                if let BookLayer::Income(position) = p {
                    Some(position.margin_used)
                } else {
                    None
                }
            })
            .sum();
        if spread_risk.is_zero() {
            Decimal::ZERO
        } else {
            shock_payout / spread_risk
        }
    }
}

/// --------------------------- QUICK TEST DRIVER ---------------------------

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    #[test]
    fn example_portfolio_json_roundtrip() {
        let demo = Portfolio {
            cash: Decimal::from_f64(100_000.0).unwrap(),
            positions: vec![],
        };
        let json = serde_json::to_string_pretty(&demo).unwrap();
        let decoded: Portfolio = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.cash, Decimal::from_f64(100_000.0).unwrap());
    }
}
