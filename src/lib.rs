pub mod domain;
mod error;
mod ibkr;
mod repo;
mod ui;

pub use error::{Error, Result};

pub use crate::domain::Position;
pub use repo::Repo;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BookLayer {
    Income,    // credit spreads, theta engines
    Insurance, // convex tail-risk hedges
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Greeks {
    pub delta: Decimal,
    pub vega: Decimal,
    pub theta: Decimal,
    pub gamma: Decimal,
    pub rho: Decimal,
}

impl Default for Greeks {
    fn default() -> Self {
        Self {
            delta: Decimal::ZERO,
            vega: Decimal::ZERO,
            theta: Decimal::ZERO,
            gamma: Decimal::ZERO,
            rho: Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub cash: Decimal,
    pub positions: Vec<Position>,
}

/// --------------------------- AGGREGATION HELPERS ---------------------------

impl Portfolio {
    /// Net portfolio Greeks (beta-weighted later if desired)
    pub fn net_greeks(&self) -> Greeks {
        self.positions.iter().flat_map(|p| &p.legs).fold(
            Greeks {
                delta: Decimal::try_from(0.0).unwrap(),
                vega: Decimal::try_from(0.0).unwrap(),
                theta: Decimal::try_from(0.0).unwrap(),
                gamma: Decimal::try_from(0.0).unwrap(),
                rho: Decimal::try_from(0.0).unwrap(),
            },
            |mut acc, leg| {
                acc.delta += leg.greeks.delta * Decimal::from_i32(leg.quantity).unwrap();
                acc.vega += leg.greeks.vega * Decimal::from_i32(leg.quantity).unwrap();
                acc.theta += leg.greeks.theta * Decimal::from_i32(leg.quantity).unwrap();
                acc.gamma += leg.greeks.gamma * Decimal::from_i32(leg.quantity).unwrap();
                acc
            },
        )
    }

    /// Total margin in use
    pub fn margin_used(&self) -> Decimal {
        self.positions
            .iter()
            .map(|p| p.margin_used)
            .sum::<Decimal>()
    }

    /// Convexity-coverage ratio (stub: replace `shock_payout` with real pricer)
    pub fn convexity_ratio(&self, shock_payout: Decimal) -> Decimal {
        let spread_risk: Decimal = self
            .positions
            .iter()
            .filter(|p| p.book_layer == BookLayer::Income)
            .map(|p| p.margin_used)
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
