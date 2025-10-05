use crate::types::JupiterQuote;

#[derive(Debug, Clone)]
pub struct SwapRequest {
    pub quote: JupiterQuote,
    pub user_public_key: String,
    pub wrap_and_unwrap_sol: bool,
    pub dynamic_compute_unit_limit: bool,
}

impl SwapRequest {
    pub fn new(quote: JupiterQuote, user_public_key: impl Into<String>) -> Self {
        Self {
            quote,
            user_public_key: user_public_key.into(),
            wrap_and_unwrap_sol: true,
            dynamic_compute_unit_limit: true,
        }
    }
}
