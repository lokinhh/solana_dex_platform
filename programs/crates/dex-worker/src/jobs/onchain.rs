use anyhow::Result;
use dex_solana::{SolanaConfig, SolanaRpc};
use onchain_indexer::OnchainWatcher;

use crate::config::WorkerConfig;
use crate::output::WorkerOutput;

#[derive(serde::Serialize)]
struct OnchainPollData {
    events: Vec<onchain_indexer::CopyLeaderEvent>,
    activities: Vec<onchain_indexer::LeaderActivity>,
}

pub async fn run_onchain_poll(config: &WorkerConfig, leaders: &[String]) -> Result<()> {
    let solana_config = SolanaConfig {
        cluster: dex_solana::Cluster::Devnet,
        rpc_url: config.solana_rpc_url.clone(),
        paper_trading: config.paper_trading,
        max_trade_sol: config.max_trade_sol,
    };

    let rpc = SolanaRpc::new(solana_config);
    let watcher = OnchainWatcher::new(rpc);
    let events = watcher.poll_leaders(leaders).await?;
    let activities = OnchainWatcher::expand_events(&events);

    WorkerOutput::success(
        "onchain_poll",
        OnchainPollData { events, activities },
    )
    .print_json()
}
