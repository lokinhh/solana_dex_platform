use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderActivity {
    pub leader: String,
    pub signature: String,
    pub mint: Option<String>,
    pub symbol: Option<String>,
    pub side: String,
    pub amount_sol: f64,
    pub detected_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyLeaderEvent {
    pub leader: String,
    pub new_signatures: Vec<String>,
    pub latest_signature: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub signature: Option<String>,
    pub fee_payer: Option<String>,
    pub leader_address: Option<String>,
    pub account: Option<String>,
    pub mint: Option<String>,
    pub symbol: Option<String>,
    pub side: Option<String>,
    pub amount_sol: Option<f64>,
}

impl WebhookPayload {
    pub fn leader(&self) -> Option<String> {
        self.fee_payer
            .clone()
            .or_else(|| self.leader_address.clone())
            .or_else(|| self.account.clone())
    }
}
