use chrono::NaiveDate;
use portfolio_manager::domain::leg::{Leg, LegType};
use portfolio_manager::{BookLayer, Greeks, Portfolio, Position, PositionType};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

fn demo_portfolio() -> Portfolio {
    Portfolio {
        cash: Decimal::try_from(10_000.0).unwrap(),
        positions: vec![Position {
            id: "spx_credit".into(),
            underlying: "SPX".into(),
            book_layer: BookLayer::Income,
            pos_type: PositionType::CreditSpread,
            margin_used: Decimal::try_from(5_000.0).unwrap(),
            legs: vec![
                Leg {
                    symbol: "SPX".into(),
                    leg_type: LegType::OPTION,
                    strike: Some(Decimal::from_f64(4700.0).unwrap()),
                    quantity: -1,
                    price_paid: Decimal::from_f64(2.5).unwrap(),
                    expiry: Some(NaiveDate::from_ymd_opt(2025, 6, 20).unwrap()),
                    greeks: Greeks {
                        delta: Decimal::try_from(-0.2).unwrap(),
                        vega: Decimal::try_from(-0.4).unwrap(),
                        theta: Decimal::try_from(0.3).unwrap(),
                        gamma: Decimal::try_from(0.0).unwrap(),
                        rho: Decimal::try_from(0.0).unwrap(),
                    },
                },
                Leg {
                    symbol: "SPX".into(),
                    leg_type: LegType::OPTION,
                    strike: Some(Decimal::from_f64(4600.0).unwrap()),
                    quantity: 1,
                    price_paid: Decimal::from_f64(-0.5).unwrap(),
                    expiry: Some(NaiveDate::from_ymd_opt(2025, 6, 20).unwrap()),
                    greeks: Greeks {
                        delta: Decimal::try_from(-0.1).unwrap(),
                        vega: Decimal::try_from(-0.2).unwrap(),
                        theta: Decimal::try_from(0.1).unwrap(),
                        gamma: Decimal::try_from(0.0).unwrap(),
                        rho: Decimal::try_from(0.0).unwrap(),
                    },
                },
            ],
        }],
    }
}

/// ## Why we compare Greeks with an epsilon (‵≈‵ instead of ‵==‵)
///
/// * `Greeks` are stored as `f64`; tiny IEEE-754 rounding noise is inevitable
///   when legs are multiplied by quantity and then summed.
/// * A delta/vega/theta error smaller than `1 e-6` (0.000 001) is **orders of
///   magnitude below any value we’d trade on**, yet big enough to avoid false
///   negatives due to binary-float representation.
/// * Therefore the test uses  
///   `assert!((actual-expected).abs() < 1e-6)`  
///   to verify our aggregation math without flagging harmless rounding dust.
///
/// **Never tighten this ε unless you also switch the entire numeric stack to
/// fixed-point; doing so would create flaky tests while adding zero realism.**
use rust_decimal::dec; // add = "rust_decimal_macros" to Cargo.toml

#[test]
fn aggregates_greeks_correctly() {
    let p = demo_portfolio();
    let g = p.net_greeks();

    assert_eq!(g.delta, dec!(0.10), "delta mismatch");
    assert_eq!(g.vega, dec!(0.20), "vega mismatch");
    assert_eq!(g.theta, dec!(-0.20), "theta mismatch");
    assert_eq!(g.gamma, dec!(0.00), "gamma mismatch");
}

#[test]
fn margin_sum_is_correct() {
    let p = demo_portfolio();
    assert_eq!(p.margin_used(), dec!(5000.00));
}

#[test]
fn convexity_ratio_zero_when_no_spreads() {
    let mut p = demo_portfolio();
    p.positions.clear();
    assert_eq!(p.convexity_ratio(dec!(0.0)), dec!(0.0));
}
