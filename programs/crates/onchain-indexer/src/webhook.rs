use crate::types::{LeaderActivity, WebhookPayload};

pub struct WebhookNormalizer;

impl WebhookNormalizer {
    pub fn from_json(value: serde_json::Value) -> WebhookPayload {
        serde_json::from_value(value).unwrap_or_default()
    }

    pub fn to_activity(payload: &WebhookPayload) -> Option<LeaderActivity> {
        let leader = payload.leader()?;
        let signature = payload.signature.clone()?;

        Some(LeaderActivity {
            leader,
            signature,
            mint: payload.mint.clone(),
            symbol: payload.symbol.clone(),
            side: payload.side.clone().unwrap_or_else(|| "buy".into()),
            amount_sol: payload.amount_sol.unwrap_or(0.05),
            detected_at: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn batch_from_array(values: Vec<serde_json::Value>) -> Vec<LeaderActivity> {
        values
            .into_iter()
            .map(Self::from_json)
            .filter_map(|p| Self::to_activity(&p))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_helius_style_payload() {
        let payload = WebhookNormalizer::from_json(json!({
            "signature": "abc123",
            "feePayer": "LeaderWallet111111111111111111111111111111",
            "mint": "PumpFunDemoMint1111111111111111111111111111",
            "side": "buy",
            "amountSol": 0.1
        }));
        let activity = WebhookNormalizer::to_activity(&payload).unwrap();
        assert_eq!(activity.side, "buy");
        assert_eq!(activity.amount_sol, 0.1);
    }
}
