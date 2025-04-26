mod repo;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
pub enum BookLayer {
    Income,    // credit spreads, theta engines
    Insurance, // convex tail-risk hedges
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Greeks {
    pub delta: f64,
    pub vega:  f64,
    pub theta: f64,
    pub gamma: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leg {
    pub leg_type: LegType,
    pub strike:   f64,
    pub quantity: i32,      // negative = short
    pub premium:  f64,      // +collected, –paid
    pub expiry:   NaiveDate,
    pub greeks:   Greeks,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LegType {
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id:           String,
    pub underlying:   String,      // "SPY", "QQQ", etc.
    pub book_layer:   BookLayer,
    pub pos_type:     PositionType,
    pub legs:         Vec<Leg>,
    pub margin_used:  f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub cash:       f64,
    pub positions:  Vec<Position>,
}

/// --------------------------- AGGREGATION HELPERS ---------------------------

impl Portfolio {
    /// Net portfolio Greeks (beta-weighted later if desired)
    pub fn net_greeks(&self) -> Greeks {
        self.positions.iter().flat_map(|p| &p.legs).fold(
            Greeks { delta: 0.0, vega: 0.0, theta: 0.0, gamma: 0.0 },
            |mut acc, leg| {
                acc.delta += leg.greeks.delta * leg.quantity as f64;
                acc.vega  += leg.greeks.vega  * leg.quantity as f64;
                acc.theta += leg.greeks.theta * leg.quantity as f64;
                acc.gamma += leg.greeks.gamma * leg.quantity as f64;
                acc
            },
        )
    }

    /// Total margin in use
    pub fn margin_used(&self) -> f64 {
        self.positions.iter().map(|p| p.margin_used).sum()
    }

    /// Convexity-coverage ratio (stub: replace `shock_payout` with real pricer)
    pub fn convexity_ratio(&self, shock_payout: f64) -> f64 {
        let spread_risk: f64 = self
            .positions
            .iter()
            .filter(|p| p.book_layer == BookLayer::Income)
            .map(|p| p.margin_used)
            .sum();
        if spread_risk == 0.0 { 0.0 } else { shock_payout / spread_risk }
    }
}

/// --------------------------- QUICK TEST DRIVER ---------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_portfolio_json_roundtrip() {
        let demo = Portfolio {
            cash: 100_000.0,
            positions: vec![],
        };
        let json = serde_json::to_string_pretty(&demo).unwrap();
        let decoded: Portfolio = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.cash, 100_000.0);
    }
}

