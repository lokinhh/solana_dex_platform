use crate::idempotency::{build_idempotency_key, IdempotencyStore};
use crate::subscription::SubscriptionStore;
use crate::types::{CopyMirrorResult, CopySubscriptionRecord, LeaderTradeEvent};

#[derive(Debug)]
pub enum CopyEngineError {
    InvalidSizePct,
    Duplicate,
    NoSubscriptions,
}

pub struct CopyTradeEngine {
    subscriptions: SubscriptionStore,
    idempotency: IdempotencyStore,
}

impl Default for CopyTradeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CopyTradeEngine {
    pub fn new() -> Self {
        Self {
            subscriptions: SubscriptionStore::new(),
            idempotency: IdempotencyStore::new(),
        }
    }

    pub fn subscribe(&self, record: CopySubscriptionRecord) -> CopySubscriptionRecord {
        self.subscriptions.upsert(record.clone());
        record
    }

    pub fn list_subscriptions(&self, user_id: &str) -> Vec<CopySubscriptionRecord> {
        self.subscriptions.list_for_user(user_id)
    }

    pub fn scale_amount(amount_sol: f64, size_pct: f64) -> Result<f64, CopyEngineError> {
        if size_pct <= 0.0 || size_pct > 100.0 {
            return Err(CopyEngineError::InvalidSizePct);
        }
        Ok((amount_sol * size_pct) / 100.0)
    }

    pub fn handle_leader_activity(
        &self,
        event: &LeaderTradeEvent,
    ) -> Result<Vec<CopyMirrorResult>, CopyEngineError> {
        let subs = self.subscriptions.active_for_leader(&event.leader_address);
        if subs.is_empty() {
            return Err(CopyEngineError::NoSubscriptions);
        }

        let mut results = Vec::new();
        for sub in subs {
            if let Some(result) = self.mirror_for_subscription(&sub, event)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    fn mirror_for_subscription(
        &self,
        sub: &CopySubscriptionRecord,
        event: &LeaderTradeEvent,
    ) -> Result<Option<CopyMirrorResult>, CopyEngineError> {
        let idempotency_key = build_idempotency_key(&event.signature, &sub.user_id);
        if let Some(cached) = self.idempotency.get(&idempotency_key) {
            return Ok(Some(cached));
        }

        let scaled = Self::scale_amount(event.amount_sol, sub.size_pct)?;

        let result = CopyMirrorResult {
            subscription_id: sub.id.clone(),
            leader_signature: event.signature.clone(),
            scaled_amount_sol: scaled,
            idempotency_key: idempotency_key.clone(),
            follower_public_key: sub.follower_public_key.clone(),
            mint: event.mint_or_default(),
            symbol: event.symbol_or_default(),
            side: event.side_or_buy(),
        };

        self.idempotency.set(idempotency_key, result.clone());
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CopySubscriptionRecord;

    fn sample_sub() -> CopySubscriptionRecord {
        CopySubscriptionRecord {
            id: "u1:leader".into(),
            user_id: "u1".into(),
            leader_address: "Leader1111111111111111111111111111111111".into(),
            follower_wallet_id: "w1".into(),
            follower_public_key: "Follower1111111111111111111111111111111".into(),
            size_pct: 50.0,
            active: true,
        }
    }

    #[test]
    fn scales_copy_size() {
        assert_eq!(CopyTradeEngine::scale_amount(1.0, 50.0).unwrap(), 0.5);
    }

    #[test]
    fn mirrors_leader_trade() {
        let engine = CopyTradeEngine::new();
        engine.subscribe(sample_sub());
        let results = engine
            .handle_leader_activity(&LeaderTradeEvent {
                leader_address: "Leader1111111111111111111111111111111111".into(),
                signature: "sig1".into(),
                mint: None,
                symbol: Some("PEPE2".into()),
                side: "buy".into(),
                amount_sol: 0.1,
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scaled_amount_sol, 0.05);
    }

    #[test]
    fn idempotency_prevents_duplicates() {
        let engine = CopyTradeEngine::new();
        engine.subscribe(sample_sub());
        let event = LeaderTradeEvent {
            leader_address: "Leader1111111111111111111111111111111111".into(),
            signature: "sig2".into(),
            mint: None,
            symbol: None,
            side: "buy".into(),
            amount_sol: 0.2,
        };
        let first = engine.handle_leader_activity(&event).unwrap();
        let second = engine.handle_leader_activity(&event).unwrap();
        assert_eq!(first[0].idempotency_key, second[0].idempotency_key);
        assert_eq!(engine.idempotency.len(), 1);
    }
}
