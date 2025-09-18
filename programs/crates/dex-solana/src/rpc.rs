use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::config::RpcSignatureStatusConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::config::SolanaConfig;
use crate::error::SolanaError;
use crate::paper::PaperLedger;
use crate::types::{BalanceSnapshot, SendResult, SignatureInfo, TradeSide};

pub struct SolanaRpc {
    config: SolanaConfig,
    client: Option<Arc<RpcClient>>,
    paper: PaperLedger,
}

impl SolanaRpc {
    pub fn new(config: SolanaConfig) -> Self {
        let client = if config.paper_trading {
            None
        } else {
            Some(Arc::new(RpcClient::new_with_commitment(
                config.rpc_url.clone(),
                CommitmentConfig::confirmed(),
            )))
        };

        Self {
            config,
            client,
            paper: PaperLedger::new(),
        }
    }

    pub fn config(&self) -> &SolanaConfig {
        &self.config
    }

    pub fn is_paper(&self) -> bool {
        self.config.paper_trading
    }

    pub fn mode(&self) -> &'static str {
        if self.is_paper() { "paper" } else { "live" }
    }

    pub fn paper_ledger(&self) -> &PaperLedger {
        &self.paper
    }

    pub async fn get_balance_sol(&self, pubkey: &str) -> Result<f64, SolanaError> {
        if self.is_paper() {
            return Ok(self.paper.get_balance_sol(pubkey));
        }

        let key = Pubkey::from_str(pubkey)
            .map_err(|e| SolanaError::InvalidPubkey(e.to_string()))?;
        let client = self.client.as_ref().ok_or_else(|| {
            SolanaError::Rpc("rpc client not initialized".into())
        })?;
        let lamports = client
            .get_balance(&key)
            .await
            .map_err(|e| SolanaError::Rpc(e.to_string()))?;
        Ok(lamports as f64 / 1_000_000_000.0)
    }

    pub async fn get_balance_snapshot(&self, pubkey: &str) -> Result<BalanceSnapshot, SolanaError> {
        if self.is_paper() {
            let sol = self.paper.get_balance_sol(pubkey);
            return Ok(BalanceSnapshot {
                pubkey: pubkey.to_string(),
                lamports: (sol * 1_000_000_000.0) as u64,
                sol,
            });
        }

        let key = Pubkey::from_str(pubkey)
            .map_err(|e| SolanaError::InvalidPubkey(e.to_string()))?;
        let client = self.client.as_ref().ok_or_else(|| {
            SolanaError::Rpc("rpc client not initialized".into())
        })?;
        let lamports = client
            .get_balance(&key)
            .await
            .map_err(|e| SolanaError::Rpc(e.to_string()))?;
        Ok(BalanceSnapshot {
            pubkey: pubkey.to_string(),
            lamports,
            sol: lamports as f64 / 1_000_000_000.0,
        })
    }

    pub async fn get_recent_signatures(
        &self,
        pubkey: &str,
        limit: usize,
    ) -> Result<Vec<SignatureInfo>, SolanaError> {
        if self.is_paper() {
            return Ok(vec![SignatureInfo {
                signature: format!("paper-sig-{}", chrono::Utc::now().timestamp_millis()),
                slot: 0,
                block_time: Some(chrono::Utc::now().timestamp()),
                err: None,
            }]);
        }

        let key = Pubkey::from_str(pubkey)
            .map_err(|e| SolanaError::InvalidPubkey(e.to_string()))?;
        let client = self.client.as_ref().ok_or_else(|| {
            SolanaError::Rpc("rpc client not initialized".into())
        })?;

        let sigs = client
            .get_signatures_for_address(&key)
            .await
            .map_err(|e| SolanaError::Rpc(e.to_string()))?;

        Ok(sigs
            .into_iter()
            .take(limit)
            .map(|row| SignatureInfo {
                signature: row.signature.to_string(),
                slot: row.slot,
                block_time: row.block_time,
                err: row.err.map(|e| format!("{e:?}")),
            })
            .collect())
    }

    pub async fn paper_swap(
        &self,
        wallet: &str,
        amount_sol: f64,
        side: TradeSide,
    ) -> Result<String, SolanaError> {
        if amount_sol > self.config.max_trade_sol {
            return Err(SolanaError::InvalidAmount(format!(
                "max trade is {} SOL",
                self.config.max_trade_sol
            )));
        }
        let result = self.paper.swap(wallet, amount_sol, side)?;
        debug!(target: "dex_solana", signature = %result.signature, "paper_swap");
        Ok(result.signature)
    }

    pub async fn send_raw_transaction_base64(
        &self,
        serialized_base64: &str,
    ) -> Result<SendResult, SolanaError> {
        if self.is_paper() {
            let signature = format!("jup-paper-{}", chrono::Utc::now().timestamp_millis());
            return Ok(SendResult {
                signature,
                status: "confirmed".into(),
                mode: "paper".into(),
            });
        }

        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let bytes = STANDARD
            .decode(serialized_base64)
            .map_err(|e| SolanaError::Transaction(e.to_string()))?;
        let tx: Transaction = bincode::deserialize(&bytes)
            .map_err(|e| SolanaError::Transaction(e.to_string()))?;

        let client = self.client.as_ref().ok_or_else(|| {
            SolanaError::Rpc("rpc client not initialized".into())
        })?;
        let signature = client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| SolanaError::Transaction(e.to_string()))?;

        Ok(SendResult {
            signature: signature.to_string(),
            status: "confirmed".into(),
            mode: "live".into(),
        })
    }

    pub async fn confirm_signature(&self, signature: &str) -> Result<bool, SolanaError> {
        if self.is_paper() {
            return Ok(true);
        }

        let sig = Signature::from_str(signature)
            .map_err(|e| SolanaError::Transaction(e.to_string()))?;
        let client = self.client.as_ref().ok_or_else(|| {
            SolanaError::Rpc("rpc client not initialized".into())
        })?;

        let statuses = client
            .get_signature_statuses_with_history(&[sig], RpcSignatureStatusConfig {
                search_transaction_history: true,
            })
            .await
            .map_err(|e| SolanaError::Rpc(e.to_string()))?;

        Ok(statuses
            .value
            .first()
            .and_then(|s| s.as_ref())
            .map(|s| s.err.is_none())
            .unwrap_or(false))
    }

    pub fn validate_amount_sol(&self, amount_sol: f64) -> Result<(), SolanaError> {
        if amount_sol <= 0.0 {
            return Err(SolanaError::InvalidAmount("must be positive".into()));
        }
        if amount_sol > self.config.max_trade_sol {
            return Err(SolanaError::InvalidAmount(format!(
                "exceeds MAX_TRADE_SOL ({})",
                self.config.max_trade_sol
            )));
        }
        Ok(())
    }
}

impl Clone for SolanaRpc {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            paper: PaperLedger::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn paper_balance_defaults() {
        let rpc = SolanaRpc::new(SolanaConfig::paper());
        let sol = rpc.get_balance_sol("demo-wallet").await.unwrap();
        assert!(sol >= 10.0);
    }

    #[tokio::test]
    async fn paper_swap_returns_signature() {
        let rpc = SolanaRpc::new(SolanaConfig::paper());
        let sig = rpc
            .paper_swap("demo-wallet", 0.1, TradeSide::Buy)
            .await
            .unwrap();
        assert!(sig.starts_with("paper-"));
    }
}
