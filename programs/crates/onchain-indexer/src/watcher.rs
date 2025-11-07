use dex_solana::SolanaRpc;
use tracing::{debug, warn};

use crate::cursor::LeaderCursorStore;
use crate::types::{CopyLeaderEvent, LeaderActivity};

#[derive(Debug, Clone)]
pub struct OnchainWatcher {
    rpc: SolanaRpc,
    cursors: LeaderCursorStore,
    signature_limit: usize,
}

impl OnchainWatcher {
    pub fn new(rpc: SolanaRpc) -> Self {
        Self {
            rpc,
            cursors: LeaderCursorStore::new(),
            signature_limit: 5,
        }
    }

    pub fn with_signature_limit(mut self, limit: usize) -> Self {
        self.signature_limit = limit;
        self
    }

    pub fn cursor_store(&self) -> &LeaderCursorStore {
        &self.cursors
    }

    pub async fn poll_leader(&self, leader: &str) -> Result<Option<CopyLeaderEvent>, dex_solana::SolanaError> {
        if self.rpc.is_paper() {
            return Ok(None);
        }

        let sigs = self
            .rpc
            .get_recent_signatures(leader, self.signature_limit)
            .await?;

        if sigs.is_empty() {
            return Ok(None);
        }

        let latest = sigs[0].signature.clone();
        let previous = self.cursors.get(leader);

        if previous.as_deref() == Some(latest.as_str()) {
            return Ok(None);
        }

        let mut new_signatures = Vec::new();
        if let Some(prev) = previous {
            for row in &sigs {
                if row.signature == prev {
                    break;
                }
                new_signatures.push(row.signature.clone());
            }
            new_signatures.reverse();
        }

        self.cursors.set(leader, latest.clone());

        if new_signatures.is_empty() && previous.is_some() {
            return Ok(None);
        }

        debug!(target: "onchain_indexer", leader, count = new_signatures.len(), "new_signatures");

        Ok(Some(CopyLeaderEvent {
            leader: leader.to_string(),
            new_signatures,
            latest_signature: latest,
        }))
    }

    pub async fn poll_leaders(
        &self,
        leaders: &[String],
    ) -> Result<Vec<CopyLeaderEvent>, dex_solana::SolanaError> {
        let mut events = Vec::new();
        for leader in leaders {
            match self.poll_leader(leader).await {
                Ok(Some(event)) => events.push(event),
                Ok(None) => {}
                Err(err) => {
                    warn!(target: "onchain_indexer", leader, error = %err, "poll_failed");
                }
            }
        }
        Ok(events)
    }

    pub fn activity_from_signature(leader: &str, signature: &str) -> LeaderActivity {
        LeaderActivity {
            leader: leader.to_string(),
            signature: signature.to_string(),
            mint: None,
            symbol: None,
            side: "buy".into(),
            amount_sol: 0.05,
            detected_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn expand_events(events: &[CopyLeaderEvent]) -> Vec<LeaderActivity> {
        events
            .iter()
            .flat_map(|event| {
                event
                    .new_signatures
                    .iter()
                    .map(|sig| Self::activity_from_signature(&event.leader, sig))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dex_solana::SolanaConfig;

    #[tokio::test]
    async fn paper_mode_skips_poll() {
        let watcher = OnchainWatcher::new(SolanaRpc::new(SolanaConfig::paper()));
        let result = watcher.poll_leader("leader").await.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn expands_signature_events() {
        let events = vec![CopyLeaderEvent {
            leader: "L".into(),
            new_signatures: vec!["s1".into(), "s2".into()],
            latest_signature: "s2".into(),
        }];
        let activities = OnchainWatcher::expand_events(&events);
        assert_eq!(activities.len(), 2);
    }
}
