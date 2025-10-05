use serde::{Deserialize, Serialize};

use crate::SOL_MINT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JupiterMode {
    Paper,
    Live,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub price_impact_pct: f64,
    pub slippage_bps: u16,
    pub mode: JupiterMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_plan: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTransaction {
    pub swap_transaction: String,
    pub mode: JupiterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

impl TradeSide {
    pub fn parse(value: &str) -> Result<Self, crate::error::JupiterError> {
        match value.to_lowercase().as_str() {
            "buy" => Ok(Self::Buy),
            "sell" => Ok(Self::Sell),
            other => Err(crate::error::JupiterError::InvalidSide(other.into())),
        }
    }
}

pub fn resolve_mints(side: TradeSide, mint: &str) -> (String, String) {
    match side {
        TradeSide::Buy => (SOL_MINT.to_string(), mint.to_string()),
        TradeSide::Sell => (mint.to_string(), SOL_MINT.to_string()),
    }
}
