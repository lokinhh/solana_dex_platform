use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;

use crate::error::SolanaError;
use crate::types::{PaperSwapResult, TradeSide};

const DEFAULT_PAPER_BALANCE_SOL: f64 = 10.0;

#[derive(Debug, Default)]
pub struct PaperLedger {
    balances: Mutex<HashMap<String, f64>>,
    counter: Mutex<u64>,
}

impl PaperLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_balance_sol(&self, wallet: &str) -> f64 {
        self.balances
            .lock()
            .expect("paper ledger lock")
            .get(wallet)
            .copied()
            .unwrap_or(DEFAULT_PAPER_BALANCE_SOL)
    }

    pub fn set_balance_sol(&self, wallet: &str, amount: f64) -> Result<(), SolanaError> {
        if amount < 0.0 {
            return Err(SolanaError::Paper("balance cannot be negative".into()));
        }
        self.balances
            .lock()
            .expect("paper ledger lock")
            .insert(wallet.to_string(), amount);
        Ok(())
    }

    pub fn swap(
        &self,
        wallet: &str,
        amount_sol: f64,
        side: TradeSide,
    ) -> Result<PaperSwapResult, SolanaError> {
        if amount_sol <= 0.0 {
            return Err(SolanaError::InvalidAmount(
                "amount must be positive".into(),
            ));
        }

        let mut balances = self.balances.lock().expect("paper ledger lock");
        let balance = balances
            .entry(wallet.to_string())
            .or_insert(DEFAULT_PAPER_BALANCE_SOL);

        match side {
            TradeSide::Buy => {
                if *balance < amount_sol {
                    return Err(SolanaError::InsufficientBalance {
                        have: *balance,
                        need: amount_sol,
                    });
                }
                *balance -= amount_sol;
            }
            TradeSide::Sell => {
                *balance += amount_sol;
            }
        }

        let mut counter = self.counter.lock().expect("paper counter lock");
        *counter += 1;
        let signature = format!(
            "paper-{}-{:x}",
            Utc::now().timestamp_millis(),
            *counter
        );

        Ok(PaperSwapResult {
            signature,
            wallet: wallet.to_string(),
            side,
            amount_sol,
            mode: "paper".into(),
        })
    }

    pub fn all_balances(&self) -> HashMap<String, f64> {
        self.balances.lock().expect("paper ledger lock").clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_reduces_balance() {
        let ledger = PaperLedger::new();
        let wallet = "wallet123";
        ledger.set_balance_sol(wallet, 1.0).unwrap();
        ledger
            .swap(wallet, 0.25, TradeSide::Buy)
            .expect("swap should succeed");
        assert_eq!(ledger.get_balance_sol(wallet), 0.75);
    }

    #[test]
    fn sell_increases_balance() {
        let ledger = PaperLedger::new();
        let wallet = "wallet456";
        ledger.swap(wallet, 0.1, TradeSide::Sell).unwrap();
        assert_eq!(ledger.get_balance_sol(wallet), DEFAULT_PAPER_BALANCE_SOL + 0.1);
    }

    #[test]
    fn rejects_overdraft() {
        let ledger = PaperLedger::new();
        let err = ledger.swap("w", 100.0, TradeSide::Buy).unwrap_err();
        assert!(matches!(err, SolanaError::InsufficientBalance { .. }));
    }
}
