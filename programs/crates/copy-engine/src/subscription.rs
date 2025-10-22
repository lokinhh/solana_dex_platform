use std::collections::HashMap;
use std::sync::RwLock;

use crate::types::CopySubscriptionRecord;

#[derive(Debug, Default)]
pub struct SubscriptionStore {
    subs: RwLock<HashMap<String, CopySubscriptionRecord>>,
}

impl SubscriptionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&self, record: CopySubscriptionRecord) {
        self.subs
            .write()
            .expect("sub write lock")
            .insert(record.id.clone(), record);
    }

    pub fn list_all(&self) -> Vec<CopySubscriptionRecord> {
        self.subs.read().expect("sub read lock").values().cloned().collect()
    }

    pub fn list_for_user(&self, user_id: &str) -> Vec<CopySubscriptionRecord> {
        self.list_all()
            .into_iter()
            .filter(|s| s.user_id == user_id)
            .collect()
    }

    pub fn active_for_leader(&self, leader: &str) -> Vec<CopySubscriptionRecord> {
        self.list_all()
            .into_iter()
            .filter(|s| s.active && s.leader_address == leader)
            .collect()
    }
}
