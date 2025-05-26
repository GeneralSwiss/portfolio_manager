use std::str::FromStr;

use crate::{ExpirationDateParseErrorKind, Greeks};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Leg {
    Stock {
        symbol: String,
        quantity: f64,
        greeks: Greeks,
    },
    Option {
        symbol: String,
        underlying_symbol: String,
        quantity: f64,
        strike: Decimal,
        expiry: ExpirationDate,
        greeks: Greeks,
    },
}

impl Leg {
    pub fn stock(symbol: String, quantity: f64) -> Self {
        Self::Stock {
            symbol,
            quantity,
            greeks: Greeks::from_stock(quantity),
        }
    }

    pub fn option(
        symbol: String,
        underlying_symbol: String,
        quantity: f64,
        strike: Decimal,
        expiry: ExpirationDate,
        greeks: Greeks,
    ) -> Self {
        Self::Option {
            symbol,
            underlying_symbol,
            strike,
            quantity,
            expiry,
            greeks,
        }
    }

    pub fn get_underlying_symbol(&self) -> &str {
        match self {
            Leg::Stock { symbol, .. } => symbol,
            Leg::Option {
                underlying_symbol, ..
            } => underlying_symbol,
        }
    }

    pub fn greeks(&self) -> Greeks {
        match self {
            Leg::Stock { greeks, .. } => *greeks,
            Leg::Option { greeks, .. } => *greeks,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExpirationDate(NaiveDate);

impl FromStr for ExpirationDate {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.len() {
            6 => Ok(ExpirationDate(
                NaiveDate::parse_from_str(s, "%Y%m")
                    .map_err(ExpirationDateParseErrorKind::ParseError)?,
            )),
            8 => Ok(ExpirationDate(
                NaiveDate::parse_from_str(s, "%Y%m%d")
                    .map_err(ExpirationDateParseErrorKind::ParseError)?,
            )),
            _ => Err(crate::Error::MalformedExpirationDate(
                crate::ExpirationDateParseErrorKind::InvalidExpirationDateLength(s.to_string()),
            )),
        }
    }
}
