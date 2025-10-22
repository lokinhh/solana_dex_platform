use chrono::Utc;
use pumpfun_client::PumpToken;

use crate::mention::SocialMention;
use crate::types::{SentimentScore, SourceBreakdown};

const VELOCITY_WINDOW_MS: i64 = 300_000;

#[derive(Debug, Clone)]
pub struct SentimentScorer {
    pub mention_weight: u32,
    pub velocity_weight: u32,
    pub holder_weight: u32,
    pub velocity_window_ms: i64,
}

impl Default for SentimentScorer {
    fn default() -> Self {
        Self {
            mention_weight: 8,
            velocity_weight: 12,
            holder_weight: 25,
            velocity_window_ms: VELOCITY_WINDOW_MS,
        }
    }
}

impl SentimentScorer {
    pub fn score_token(&self, token: &PumpToken, mentions: &[SocialMention]) -> SentimentScore {
        let recent: Vec<_> = mentions
            .iter()
            .filter(|m| m.mint == token.mint)
            .cloned()
            .collect();

        let now = Utc::now().timestamp_millis();
        let velocity = recent
            .iter()
            .filter(|m| now - m.ts < self.velocity_window_ms)
            .count() as u32;

        let mention_score = (recent.len() as u32 * self.mention_weight).min(40);
        let velocity_score = (velocity * self.velocity_weight).min(35);
        let holder_score = ((token.holders / 100) as u32).min(self.holder_weight);

        let raw = mention_score + velocity_score + holder_score;
        let score = raw.min(100) as u8;

        SentimentScore {
            mint: token.mint.clone(),
            symbol: token.symbol.clone(),
            score,
            mentions: recent.len() as u32,
            velocity,
            sources: SourceBreakdown {
                twitter: recent
                    .iter()
                    .filter(|m| matches!(m.source, crate::mention::MentionSource::Twitter))
                    .count() as u32,
                telegram: recent
                    .iter()
                    .filter(|m| matches!(m.source, crate::mention::MentionSource::Telegram))
                    .count() as u32,
            },
            updated_at: now,
        }
    }

    pub fn rank_tokens(
        &self,
        tokens: &[PumpToken],
        mentions: &[SocialMention],
    ) -> Vec<SentimentScore> {
        let mut scores: Vec<_> = tokens
            .iter()
            .map(|token| self.score_token(token, mentions))
            .collect();
        scores.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.velocity.cmp(&a.velocity)));
        scores
    }

    pub fn filter_actionable<'a>(
        &self,
        scores: &'a [SentimentScore],
        threshold: u8,
    ) -> Vec<&'a SentimentScore> {
        scores
            .iter()
            .filter(|s| s.is_actionable(threshold))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mention::default_mock_mentions;
    use pumpfun_client::fixtures::mock_tokens;

    #[test]
    fn scores_are_bounded() {
        let scorer = SentimentScorer::default();
        let tokens = mock_tokens();
        let scores = scorer.rank_tokens(&tokens, &default_mock_mentions());
        assert!(scores.iter().all(|s| s.score <= 100));
    }

    #[test]
    fn more_mentions_increase_score() {
        let scorer = SentimentScorer::default();
        let token = mock_tokens()
            .into_iter()
            .find(|t| t.symbol == "PEPE2")
            .unwrap();
        let base = scorer.score_token(&token, &default_mock_mentions());

        let mut extra = default_mock_mentions();
        for i in 0..5 {
            extra.push(crate::mention::SocialMention {
                source: crate::mention::MentionSource::Twitter,
                text: format!("hype {i}"),
                mint: token.mint.clone(),
                ts: chrono::Utc::now().timestamp_millis(),
            });
        }
        let boosted = scorer.score_token(&token, &extra);
        assert!(boosted.score >= base.score);
    }
}
