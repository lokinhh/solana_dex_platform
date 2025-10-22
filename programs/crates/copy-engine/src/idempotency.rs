use std::collections::HashMap;
use std::sync::RwLock;

use crate::types::CopyMirrorResult;

#[derive(Debug, Default)]
pub struct IdempotencyStore {
    cache: RwLock<HashMap<String, CopyMirrorResult>>,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<CopyMirrorResult> {
        self.cache.read().expect("idem read lock").get(key).cloned()
    }

    pub fn set(&self, key: String, value: CopyMirrorResult) {
        self.cache
            .write()
            .expect("idem write lock")
            .insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.cache.read().expect("idem read lock").len()
    }
}

pub fn build_idempotency_key(signature: &str, user_id: &str) -> String {
    format!("copy:{signature}:{user_id}")
}
