use thiserror::Error;

#[derive(Debug, Error)]
pub enum JupiterError {
    #[error("http error: {0}")]
    Http(String),
    #[error("invalid side: {0}")]
    InvalidSide(String),
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    #[error("quote failed: {0}")]
    QuoteFailed(String),
    #[error("swap build failed: {0}")]
    SwapFailed(String),
    #[error("serialization error: {0}")]
    Serde(String),
}
