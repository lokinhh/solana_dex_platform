use reqwest::Client;
use tracing::warn;

use crate::error::PumpfunError;
use crate::fixtures::{enrich_paper_token, mock_tokens};
use crate::types::PumpToken;

#[derive(Debug, Clone)]
pub struct PumpfunClient {
    base_url: String,
    paper: bool,
    http: Client,
}

impl PumpfunClient {
    pub fn new(base_url: impl Into<String>, paper: bool) -> Self {
        Self {
            base_url: base_url.into(),
            paper,
            http: Client::new(),
        }
    }

    pub fn from_env() -> Self {
        let base_url = std::env::var("PUMPFUN_API_URL")
            .unwrap_or_else(|_| "https://frontend-api.pump.fun".into());
        let paper = std::env::var("PAPER_TRADING")
            .map(|v| v != "false")
            .unwrap_or(true);
        Self::new(base_url, paper)
    }

    pub fn paper() -> Self {
        Self::new("https://frontend-api.pump.fun", true)
    }

    pub async fn list_trending(&self, limit: usize) -> Result<Vec<PumpToken>, PumpfunError> {
        if self.paper {
            let seed = chrono::Utc::now().timestamp_millis() as u64;
            return Ok(mock_tokens()
                .into_iter()
                .take(limit)
                .enumerate()
                .map(|(i, t)| enrich_paper_token(t, seed + i as u64))
                .collect());
        }

        let url = format!("{}/coins/trending?limit={}", self.base_url, limit);
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| PumpfunError::Http(e.to_string()))?;

        if !response.status().is_success() {
            warn!(target: "pumpfun", status = %response.status(), "trending fetch failed, using fixtures");
            return Ok(mock_tokens().into_iter().take(limit).collect());
        }

        let tokens: Vec<PumpToken> = response
            .json()
            .await
            .map_err(|e| PumpfunError::Serde(e.to_string()))?;

        Ok(tokens)
    }

    pub async fn get_token(&self, mint: &str) -> Result<Option<PumpToken>, PumpfunError> {
        let tokens = self.list_trending(50).await?;
        Ok(tokens.into_iter().find(|t| t.mint == mint))
    }

    pub async fn filter_by_bonding(&self, min_pct: f64, limit: usize) -> Result<Vec<PumpToken>, PumpfunError> {
        let tokens = self.list_trending(50).await?;
        Ok(tokens
            .into_iter()
            .filter(|t| t.bonding_curve_pct >= min_pct)
            .take(limit)
            .collect())
    }

    pub async fn top_by_market_cap(&self, limit: usize) -> Result<Vec<PumpToken>, PumpfunError> {
        let mut tokens = self.list_trending(50).await?;
        tokens.sort_by(|a, b| {
            b.market_cap
                .partial_cmp(&a.market_cap)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        tokens.truncate(limit);
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn paper_trending_returns_tokens() {
        let client = PumpfunClient::paper();
        let tokens = client.list_trending(3).await.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(!tokens[0].symbol.is_empty());
    }

    #[tokio::test]
    async fn get_token_finds_mint() {
        let client = PumpfunClient::paper();
        let token = client
            .get_token("PumpFunDemoMint1111111111111111111111111111")
            .await
            .unwrap();
        assert!(token.is_some());
    }
}
