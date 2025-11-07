use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct LeaderCursorStore {
    cursors: RwLock<HashMap<String, String>>,
}

impl LeaderCursorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, leader: &str) -> Option<String> {
        self.cursors
            .read()
            .expect("cursor read lock")
            .get(leader)
            .cloned()
    }

    pub fn set(&self, leader: impl Into<String>, signature: impl Into<String>) {
        self.cursors
            .write()
            .expect("cursor write lock")
            .insert(leader.into(), signature.into());
    }

    pub fn all(&self) -> HashMap<String, String> {
        self.cursors.read().expect("cursor read lock").clone()
    }

    pub fn clear(&self) {
        self.cursors.write().expect("cursor write lock").clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_retrieves_cursor() {
        let store = LeaderCursorStore::new();
        store.set("leader1", "sigA");
        assert_eq!(store.get("leader1"), Some("sigA".into()));
    }
}
