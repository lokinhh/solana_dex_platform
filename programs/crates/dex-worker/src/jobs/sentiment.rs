use anyhow::Result;
use pumpfun_client::PumpfunClient;
use sentiment_core::SentimentEngine;

use crate::config::WorkerConfig;
use crate::output::WorkerOutput;

#[derive(serde::Serialize)]
struct SentimentTickData {
    scores: Vec<sentiment_core::SentimentScore>,
    actionable: Vec<sentiment_core::SentimentScore>,
}

pub async fn run_sentiment_tick(
    config: &WorkerConfig,
    limit: usize,
    threshold: u8,
) -> Result<()> {
    let pumpfun = PumpfunClient::new(&config.pumpfun_api_url, config.paper_trading);
    let mut engine = SentimentEngine::new(pumpfun).with_trending_limit(limit);
    let snapshot = engine.tick().await?;
    let actionable = snapshot
        .scores
        .iter()
        .filter(|s| s.is_actionable(threshold))
        .cloned()
        .collect();

    WorkerOutput::success(
        "sentiment_tick",
        SentimentTickData {
            scores: snapshot.scores,
            actionable,
        },
    )
    .print_json()
}
