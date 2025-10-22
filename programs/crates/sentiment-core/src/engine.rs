use pumpfun_client::{PumpfunClient, PumpToken};

use crate::mention::{default_mock_mentions, MentionBuffer, SocialMention};
use crate::scorer::SentimentScorer;
use crate::types::{SentimentScore, SentimentSnapshot};

pub struct SentimentEngine {
    pumpfun: PumpfunClient,
    buffer: MentionBuffer,
    scorer: SentimentScorer,
    trending_limit: usize,
}

impl SentimentEngine {
    pub fn new(pumpfun: PumpfunClient) -> Self {
        Self {
            pumpfun,
            buffer: MentionBuffer::new(200),
            scorer: SentimentScorer::default(),
            trending_limit: 10,
        }
    }

    pub fn with_trending_limit(mut self, limit: usize) -> Self {
        self.trending_limit = limit;
        self
    }

    pub fn ingest_mention(&mut self, source: &str, text: &str, mint: &str) {
        self.buffer.ingest(source, text, mint);
    }

    pub fn mention_count(&self) -> usize {
        self.buffer.len()
    }

    pub async fn tick(&mut self) -> Result<SentimentSnapshot, pumpfun_client::PumpfunError> {
        let tokens = self.pumpfun.list_trending(self.trending_limit).await?;
        let mentions = self.collect_mentions();
        let scores = self.scorer.rank_tokens(&tokens, &mentions);
        Ok(SentimentSnapshot {
            scores,
            generated_at: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub async fn score_mint(&mut self, mint: &str) -> Result<Option<SentimentScore>, pumpfun_client::PumpfunError> {
        let token = self.pumpfun.get_token(mint).await?;
        let Some(token) = token else {
            return Ok(None);
        };
        let mentions = self.collect_mentions();
        Ok(Some(self.scorer.score_token(&token, &mentions)))
    }

    pub async fn top_actionable(
        &mut self,
        threshold: u8,
    ) -> Result<Vec<SentimentScore>, pumpfun_client::PumpfunError> {
        let snapshot = self.tick().await?;
        Ok(snapshot
            .scores
            .into_iter()
            .filter(|s| s.is_actionable(threshold))
            .collect())
    }

    fn collect_mentions(&self) -> Vec<SocialMention> {
        let mut mentions = default_mock_mentions();
        mentions.extend(self.buffer.all());
        mentions
    }

    pub fn score_tokens_offline(
        &self,
        tokens: &[PumpToken],
        mentions: &[SocialMention],
    ) -> Vec<SentimentScore> {
        self.scorer.rank_tokens(tokens, mentions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tick_produces_scores() {
        let mut engine = SentimentEngine::new(PumpfunClient::paper());
        let snapshot = engine.tick().await.unwrap();
        assert!(!snapshot.scores.is_empty());
    }

    #[tokio::test]
    async fn ingest_increases_buffer() {
        let mut engine = SentimentEngine::new(PumpfunClient::paper());
        engine.ingest_mention("twitter", "moon", "mint");
        assert_eq!(engine.mention_count(), 1);
    }
}
