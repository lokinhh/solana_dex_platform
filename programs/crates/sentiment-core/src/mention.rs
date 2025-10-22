use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MentionSource {
    Twitter,
    Telegram,
    Discord,
    Other,
}

impl MentionSource {
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "twitter" | "x" => Self::Twitter,
            "telegram" => Self::Telegram,
            "discord" => Self::Discord,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMention {
    pub source: MentionSource,
    pub text: String,
    pub mint: String,
    pub ts: i64,
}

#[derive(Debug, Default)]
pub struct MentionBuffer {
    max_size: usize,
    items: VecDeque<SocialMention>,
}

impl MentionBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            items: VecDeque::new(),
        }
    }

    pub fn ingest(&mut self, source: &str, text: impl Into<String>, mint: impl Into<String>) {
        self.items.push_back(SocialMention {
            source: MentionSource::parse(source),
            text: text.into(),
            mint: mint.into(),
            ts: Utc::now().timestamp_millis(),
        });
        while self.items.len() > self.max_size {
            self.items.pop_front();
        }
    }

    pub fn all(&self) -> Vec<SocialMention> {
        self.items.iter().cloned().collect()
    }

    pub fn for_mint(&self, mint: &str) -> Vec<SocialMention> {
        self.items
            .iter()
            .filter(|m| m.mint == mint)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub fn default_mock_mentions() -> Vec<SocialMention> {
    vec![
        SocialMention {
            source: MentionSource::Twitter,
            text: "$PEPE2 going parabolic on pump.fun".into(),
            mint: "PumpFunDemoMint1111111111111111111111111111".into(),
            ts: Utc::now().timestamp_millis(),
        },
        SocialMention {
            source: MentionSource::Twitter,
            text: "aped BONKAI early LFG".into(),
            mint: "PumpFunDemoMint2222222222222222222222222222".into(),
            ts: Utc::now().timestamp_millis(),
        },
        SocialMention {
            source: MentionSource::Telegram,
            text: "WIF2 bonding curve almost done".into(),
            mint: "PumpFunDemoMint3333333333333333333333333333".into(),
            ts: Utc::now().timestamp_millis(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_respects_max_size() {
        let mut buffer = MentionBuffer::new(2);
        buffer.ingest("twitter", "a", "mint1");
        buffer.ingest("twitter", "b", "mint2");
        buffer.ingest("telegram", "c", "mint3");
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.all().last().unwrap().mint, "mint3");
    }
}
