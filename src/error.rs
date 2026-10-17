//! General-purpose error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CuacaError {
    #[error("Location error: {0}")]
    Location(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Data error: {0}")]
    Data(String),

    #[error("Unexpected error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for CuacaError {
    fn from(e: serde_json::Error) -> Self {
        CuacaError::Parse(e.to_string())
    }
}

pub type WarningResult<T> = Result<T, CuacaError>;
