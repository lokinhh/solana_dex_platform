use std::sync::Arc;

use copy_engine::CopyTradeEngine;
use dex_solana::{SolanaConfig, SolanaRpc};
use pumpfun_client::PumpfunClient;
use sentiment_core::SentimentEngine;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub sentiment: Arc<Mutex<SentimentEngine>>,
    pub copy_trade: Arc<CopyTradeEngine>,
    pub solana: Arc<SolanaRpc>,
    pub pumpfun: Arc<PumpfunClient>,
}

impl AppState {
    pub fn new() -> Self {
        let paper = std::env::var("PAPER_TRADING")
            .map(|v| v != "false")
            .unwrap_or(true);

        Self {
            sentiment: Arc::new(Mutex::new(SentimentEngine::new(PumpfunClient::new(
                std::env::var("PUMPFUN_API_URL")
                    .unwrap_or_else(|_| "https://frontend-api.pump.fun".into()),
                paper,
            )))),
            copy_trade: Arc::new(CopyTradeEngine::new()),
            solana: Arc::new(SolanaRpc::new(SolanaConfig::from_env())),
            pumpfun: Arc::new(PumpfunClient::from_env()),
        }
    }
}
