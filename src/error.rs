use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Broker Client Error: {0}")]
    BrokerError(#[from] ibapi::errors::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
