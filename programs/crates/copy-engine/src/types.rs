use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopySubscriptionRecord {
    pub id: String,
    pub user_id: String,
    pub leader_address: String,
    pub follower_wallet_id: String,
    pub follower_public_key: String,
    pub size_pct: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderTradeEvent {
    pub leader_address: String,
    pub signature: String,
    pub mint: Option<String>,
    pub symbol: Option<String>,
    pub side: String,
    pub amount_sol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyMirrorRequest {
    pub subscription: CopySubscriptionRecord,
    pub event: LeaderTradeEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyMirrorResult {
    pub subscription_id: String,
    pub leader_signature: String,
    pub scaled_amount_sol: f64,
    pub idempotency_key: String,
    pub follower_public_key: String,
    pub mint: String,
    pub symbol: String,
    pub side: String,
}

impl LeaderTradeEvent {
    pub fn mint_or_default(&self) -> String {
        self.mint
            .clone()
            .unwrap_or_else(|| "PumpFunDemoMint1111111111111111111111111111".into())
    }

    pub fn symbol_or_default(&self) -> String {
        self.symbol.clone().unwrap_or_else(|| "TOKEN".into())
    }

    pub fn side_or_buy(&self) -> String {
        if self.side.is_empty() {
            "buy".into()
        } else {
            self.side.clone()
        }
    }
}
