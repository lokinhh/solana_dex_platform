use crate::types::TradeSide;

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub side: TradeSide,
    pub mint: String,
    pub amount_lamports: u64,
    pub slippage_bps: u16,
}

impl QuoteRequest {
    pub fn new(side: TradeSide, mint: impl Into<String>, amount_lamports: u64) -> Self {
        Self {
            side,
            mint: mint.into(),
            amount_lamports,
            slippage_bps: 300,
        }
    }

    pub fn with_slippage(mut self, slippage_bps: u16) -> Self {
        self.slippage_bps = slippage_bps;
        self
    }

    pub fn amount_sol(&self) -> f64 {
        self.amount_lamports as f64 / 1_000_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TradeSide;

    #[test]
    fn converts_lamports_to_sol() {
        let req = QuoteRequest::new(TradeSide::Buy, "mint", 500_000_000);
        assert_eq!(req.amount_sol(), 0.5);
    }
}
