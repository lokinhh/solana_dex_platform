use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cluster {
    Devnet,
    Testnet,
    MainnetBeta,
}

impl Cluster {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "devnet" => Some(Self::Devnet),
            "testnet" => Some(Self::Testnet),
            "mainnet-beta" | "mainnet" => Some(Self::MainnetBeta),
            _ => None,
        }
    }

    pub fn default_rpc(self) -> &'static str {
        match self {
            Self::Devnet => "https://api.devnet.solana.com",
            Self::Testnet => "https://api.testnet.solana.com",
            Self::MainnetBeta => "https://api.mainnet-beta.solana.com",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Devnet => "devnet",
            Self::Testnet => "testnet",
            Self::MainnetBeta => "mainnet-beta",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

impl TradeSide {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "buy" => Some(Self::Buy),
            "sell" => Some(Self::Sell),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub err: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub pubkey: String,
    pub lamports: u64,
    pub sol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSwapResult {
    pub signature: String,
    pub wallet: String,
    pub side: TradeSide,
    pub amount_sol: f64,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub signature: String,
    pub status: String,
    pub mode: String,
}
