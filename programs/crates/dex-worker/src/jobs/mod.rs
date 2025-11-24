mod onchain;
mod sentiment;

pub use onchain::run_onchain_poll;
pub use sentiment::run_sentiment_tick;

pub mod run_loop {
    use anyhow::Result;
    use tokio::time::{interval, Duration};
    use tracing::info;

    use crate::config::WorkerConfig;
    use super::{run_onchain_poll, run_sentiment_tick};

    pub async fn run_loop(
        config: &WorkerConfig,
        sentiment_secs: u64,
        onchain_secs: u64,
        leaders: &[String],
    ) -> Result<()> {
        let mut sentiment_timer = interval(Duration::from_secs(sentiment_secs));
        let mut onchain_timer = interval(Duration::from_secs(onchain_secs));

        info!(target: "dex_worker", "worker_loop_started");

        loop {
            tokio::select! {
                _ = sentiment_timer.tick() => {
                    run_sentiment_tick(config, 10, 70).await?;
                }
                _ = onchain_timer.tick() => {
                    if !leaders.is_empty() && !config.paper_trading {
                        run_onchain_poll(config, leaders).await?;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!(target: "dex_worker", "shutdown_signal");
                    break;
                }
            }
        }

        Ok(())
    }
}

pub use run_loop::run_loop;
