use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WorkerConfig {
    pub paper_trading: bool,
    pub solana_cluster: String,
    pub solana_rpc_url: String,
    pub jupiter_api_url: String,
    pub pumpfun_api_url: String,
    pub max_trade_sol: f64,
    pub sentiment_poll_ms: u64,
    pub onchain_poll_ms: u64,
}

impl WorkerConfig {
    pub fn from_env() -> Self {
        Self {
            paper_trading: std::env::var("PAPER_TRADING")
                .map(|v| v != "false")
                .unwrap_or(true),
            solana_cluster: std::env::var("SOLANA_CLUSTER").unwrap_or_else(|_| "devnet".into()),
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".into()),
            jupiter_api_url: std::env::var("JUPITER_API_URL")
                .unwrap_or_else(|_| "https://quote-api.jup.ag/v6".into()),
            pumpfun_api_url: std::env::var("PUMPFUN_API_URL")
                .unwrap_or_else(|_| "https://frontend-api.pump.fun".into()),
            max_trade_sol: std::env::var("MAX_TRADE_SOL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            sentiment_poll_ms: std::env::var("SENTIMENT_POLL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15_000),
            onchain_poll_ms: std::env::var("ONCHAIN_POLL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(12_000),
        }
    }
}
