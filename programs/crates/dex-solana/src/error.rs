use thiserror::Error;

#[derive(Debug, Error)]
pub enum SolanaError {
    #[error("invalid public key: {0}")]
    InvalidPubkey(String),
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("insufficient balance: have {have} need {need}")]
    InsufficientBalance { have: f64, need: f64 },
    #[error("invalid trade amount: {0}")]
    InvalidAmount(String),
    #[error("paper ledger error: {0}")]
    Paper(String),
    #[error("transaction failed: {0}")]
    Transaction(String),
}
