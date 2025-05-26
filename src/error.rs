use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Broker Client Error: {0}")]
    BrokerError(#[from] ibapi::errors::Error),

    #[error("Logging Initialization Failure")]
    LoggingInitFailure,

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unknown Leg Type: {0}")]
    UnknownLegType(String),

    #[error("Incorrectly formatted expiration date: {0} - Must be YYYYMM or YYYYMMDD")]
    MalformedExpirationDate(#[from] ExpirationDateParseErrorKind),

    #[error("Malformed Decimal: {0}")]
    MalformedDecimalError(String),
}

#[derive(Debug, Error)]
pub enum ExpirationDateParseErrorKind {
    #[error("Invalid Expiration Date Length: {0}")]
    InvalidExpirationDateLength(String),

    #[error("Unable to parse Expiration Date: {0}")]
    ParseError(#[from] chrono::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
