use crate::types::Cluster;

#[derive(Debug, Clone)]
pub struct SolanaConfig {
    pub cluster: Cluster,
    pub rpc_url: String,
    pub paper_trading: bool,
    pub max_trade_sol: f64,
}

impl SolanaConfig {
    pub fn from_env() -> Self {
        let cluster = std::env::var("SOLANA_CLUSTER")
            .ok()
            .and_then(|v| Cluster::parse(&v))
            .unwrap_or(Cluster::Devnet);
        let rpc_url = std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| cluster.default_rpc());
        let paper_trading = std::env::var("PAPER_TRADING")
            .map(|v| v != "false")
            .unwrap_or(true);
        let max_trade_sol = std::env::var("MAX_TRADE_SOL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);

        Self {
            cluster,
            rpc_url,
            paper_trading,
            max_trade_sol,
        }
    }

    pub fn paper() -> Self {
        Self {
            cluster: Cluster::Devnet,
            rpc_url: Cluster::Devnet.default_rpc().to_string(),
            paper_trading: true,
            max_trade_sol: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_defaults() {
        let cfg = SolanaConfig::paper();
        assert!(cfg.paper_trading);
        assert_eq!(cfg.cluster, Cluster::Devnet);
    }
}
