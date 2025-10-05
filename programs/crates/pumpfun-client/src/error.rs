use thiserror::Error;

#[derive(Debug, Error)]
pub enum PumpfunError {
    #[error("http error: {0}")]
    Http(String),
    #[error("token not found: {0}")]
    NotFound(String),
    #[error("serialization error: {0}")]
    Serde(String),
}
