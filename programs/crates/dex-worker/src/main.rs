mod config;
mod jobs;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use crate::config::WorkerConfig;
use crate::jobs::{run_onchain_poll, run_sentiment_tick};

#[derive(Parser)]
#[command(name = "dex-worker", about = "Rust background worker for SolDex platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run one sentiment scoring tick and print JSON snapshot.
    SentimentTick {
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long, default_value_t = 70)]
        actionable_threshold: u8,
    },
    /// Poll leader wallets for new signatures.
    OnchainPoll {
        #[arg(long, value_delimiter = ',')]
        leaders: Vec<String>,
    },
    /// Print worker configuration resolved from environment.
    Config,
    /// Continuous loop running sentiment + optional on-chain poll.
    Run {
        #[arg(long, default_value_t = 15)]
        sentiment_interval_secs: u64,
        #[arg(long, default_value_t = 12)]
        onchain_interval_secs: u64,
        #[arg(long, value_delimiter = ',')]
        leaders: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("dex_worker=info".parse()?))
        .init();

    let cli = Cli::parse();
    let config = WorkerConfig::from_env();

    match cli.command {
        Commands::SentimentTick {
            limit,
            actionable_threshold,
        } => run_sentiment_tick(&config, limit, actionable_threshold).await?,
        Commands::OnchainPoll { leaders } => run_onchain_poll(&config, &leaders).await?,
        Commands::Config => {
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        Commands::Run {
            sentiment_interval_secs,
            onchain_interval_secs,
            leaders,
        } => {
            jobs::run_loop(
                &config,
                sentiment_interval_secs,
                onchain_interval_secs,
                &leaders,
            )
            .await?;
        }
    }

    Ok(())
}
