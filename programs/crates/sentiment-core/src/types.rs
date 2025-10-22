use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    pub mint: String,
    pub symbol: String,
    pub score: u8,
    pub mentions: u32,
    pub velocity: u32,
    pub sources: SourceBreakdown,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceBreakdown {
    pub twitter: u32,
    pub telegram: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSnapshot {
    pub scores: Vec<SentimentScore>,
    pub generated_at: i64,
}

impl SentimentScore {
    pub fn is_actionable(&self, threshold: u8) -> bool {
        self.score >= threshold
    }
}
