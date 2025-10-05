pub mod client;
pub mod error;
pub mod quote;
pub mod swap;
pub mod types;

pub use client::JupiterClient;
pub use error::JupiterError;
pub use quote::QuoteRequest;
pub use swap::SwapRequest;
pub use types::{JupiterMode, JupiterQuote, SwapTransaction};

pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
