use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PumpToken {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub market_cap: f64,
    pub bonding_curve_pct: f64,
    #[serde(default)]
    pub holders: u64,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_24h: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

impl PumpToken {
    pub fn is_bonding_complete(&self) -> bool {
        self.bonding_curve_pct >= 100.0
    }

    pub fn liquidity_score(&self) -> f64 {
        let volume = self.volume_24h.unwrap_or(0.0);
        (volume / 10_000.0).min(100.0)
    }
}
