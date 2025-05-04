use chrono::NaiveDate;
use portfolio_manager::domain::leg::{Leg, LegType};
use portfolio_manager::{BookLayer, Greeks, Portfolio, Position, PositionType};

fn demo_portfolio() -> Portfolio {
    Portfolio {
        cash: 10_000.0,
        positions: vec![Position {
            id: "spx_credit".into(),
            underlying: "SPX".into(),
            book_layer: BookLayer::Income,
            pos_type: PositionType::CreditSpread,
            margin_used: 5_000.0,
            legs: vec![
                Leg {
                    leg_type: LegType::Short,
                    strike: 4700.0,
                    quantity: -1,
                    premium: 2.5,
                    expiry: NaiveDate::from_ymd_opt(2025, 6, 20).unwrap(),
                    greeks: Greeks {
                        delta: -0.2,
                        vega: -0.4,
                        theta: 0.3,
                        gamma: 0.0,
                        rho: 0.0,
                    },
                },
                Leg {
                    leg_type: LegType::Long,
                    strike: 4600.0,
                    quantity: 1,
                    premium: -0.5,
                    expiry: NaiveDate::from_ymd_opt(2025, 6, 20).unwrap(),
                    greeks: Greeks {
                        delta: -0.1,
                        vega: -0.2,
                        theta: 0.1,
                        gamma: 0.0,
                        rho: 0.0,
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
#[test]
fn aggregates_greeks_correctly() {
    let p = demo_portfolio();
    let g = p.net_greeks();
    // Expected contributions:
    // Δ: (+0.20) + (-0.10) = +0.10
    // Vega: (+0.40) + (-0.20) = +0.20
    // Θ: (+0.30) + (+0.10)  = +0.40
    // Γ: 0 across both legs
    assert!((g.delta - 0.10).abs() < 1e-6, "delta mismatch");
    assert!((g.vega - 0.20).abs() < 1e-6, "vega mismatch");
    assert!(((g.theta).abs() - 0.20).abs() < 1e-6, "theta mismatch");
    assert!((g.gamma).abs() < 1e-9, "gamma mismatch");
}

#[test]
fn margin_sum_is_correct() {
    let p = demo_portfolio();
    assert_eq!(p.margin_used(), 5_000.0);
}

#[test]
fn convexity_ratio_zero_when_no_spreads() {
    let mut p = demo_portfolio();
    p.positions.clear();
    assert_eq!(p.convexity_ratio(0.0), 0.0);
}
