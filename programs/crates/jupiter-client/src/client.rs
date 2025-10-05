use reqwest::Client;
use tracing::warn;

use crate::error::JupiterError;
use crate::quote::QuoteRequest;
use crate::swap::SwapRequest;
use crate::types::{resolve_mints, JupiterMode, JupiterQuote, SwapTransaction, TradeSide};

#[derive(Debug, Clone)]
pub struct JupiterClient {
    base_url: String,
    paper: bool,
    default_slippage_bps: u16,
    http: Client,
}

impl JupiterClient {
    pub fn new(base_url: impl Into<String>, paper: bool) -> Self {
        Self {
            base_url: base_url.into(),
            paper,
            default_slippage_bps: 300,
            http: Client::new(),
        }
    }

    pub fn from_env() -> Self {
        let base_url = std::env::var("JUPITER_API_URL")
            .unwrap_or_else(|_| "https://quote-api.jup.ag/v6".into());
        let paper = std::env::var("PAPER_TRADING")
            .map(|v| v != "false")
            .unwrap_or(true);
        let default_slippage_bps = std::env::var("DEFAULT_SLIPPAGE_BPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        Self {
            base_url,
            paper,
            default_slippage_bps,
            http: Client::new(),
        }
    }

    pub fn paper() -> Self {
        Self::new("https://quote-api.jup.ag/v6", true)
    }

    pub fn is_paper(&self) -> bool {
        self.paper
    }

    pub async fn get_quote(&self, request: QuoteRequest) -> Result<JupiterQuote, JupiterError> {
        if request.amount_lamports == 0 {
            return Err(JupiterError::InvalidAmount("zero lamports".into()));
        }

        let slippage = if request.slippage_bps == 0 {
            self.default_slippage_bps
        } else {
            request.slippage_bps
        };

        let (input_mint, output_mint) = resolve_mints(request.side, &request.mint);

        if self.paper {
            let out_amount = match request.side {
                TradeSide::Buy => request.amount_lamports.saturating_mul(1000),
                TradeSide::Sell => request.amount_lamports,
            };
            return Ok(JupiterQuote {
                input_mint,
                output_mint,
                in_amount: request.amount_lamports.to_string(),
                out_amount: out_amount.to_string(),
                price_impact_pct: 0.1,
                slippage_bps: slippage,
                mode: JupiterMode::Paper,
                route_plan: None,
            });
        }

        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.base_url, input_mint, output_mint, request.amount_lamports, slippage
        );

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| JupiterError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(JupiterError::QuoteFailed(body.chars().take(200).collect()));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| JupiterError::Serde(e.to_string()))?;

        Ok(JupiterQuote {
            input_mint: raw
                .get("inputMint")
                .and_then(|v| v.as_str())
                .unwrap_or(&input_mint)
                .to_string(),
            output_mint: raw
                .get("outputMint")
                .and_then(|v| v.as_str())
                .unwrap_or(&output_mint)
                .to_string(),
            in_amount: raw
                .get("inAmount")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            out_amount: raw
                .get("outAmount")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            price_impact_pct: raw
                .get("priceImpactPct")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .or_else(|| raw.get("priceImpactPct").and_then(|v| v.as_f64()))
                .unwrap_or(0.0),
            slippage_bps: slippage,
            mode: JupiterMode::Live,
            route_plan: raw.get("routePlan").cloned(),
        })
    }

    pub async fn build_swap_transaction(
        &self,
        request: SwapRequest,
    ) -> Result<SwapTransaction, JupiterError> {
        if self.paper {
            let payload = format!("paper-tx-{}", chrono::Utc::now().timestamp_millis());
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            return Ok(SwapTransaction {
                swap_transaction: STANDARD.encode(payload.as_bytes()),
                mode: JupiterMode::Paper,
            });
        }

        let body = serde_json::json!({
            "quoteResponse": {
                "inputMint": request.quote.input_mint,
                "outputMint": request.quote.output_mint,
                "inAmount": request.quote.in_amount,
                "outAmount": request.quote.out_amount,
                "priceImpactPct": request.quote.price_impact_pct,
                "slippageBps": request.quote.slippage_bps,
                "routePlan": request.quote.route_plan,
            },
            "userPublicKey": request.user_public_key,
            "wrapAndUnwrapSol": request.wrap_and_unwrap_sol,
            "dynamicComputeUnitLimit": request.dynamic_compute_unit_limit,
        });

        let response = self
            .http
            .post(format!("{}/swap", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| JupiterError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(JupiterError::SwapFailed(format!(
                "status {}",
                response.status()
            )));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| JupiterError::Serde(e.to_string()))?;

        let swap_transaction = raw
            .get("swapTransaction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JupiterError::SwapFailed("missing swapTransaction".into()))?
            .to_string();

        Ok(SwapTransaction {
            swap_transaction,
            mode: JupiterMode::Live,
        })
    }

    pub async fn quote_and_build(
        &self,
        side: TradeSide,
        mint: &str,
        amount_lamports: u64,
        user_public_key: &str,
    ) -> Result<(JupiterQuote, SwapTransaction), JupiterError> {
        let quote = self
            .get_quote(QuoteRequest::new(side, mint, amount_lamports))
            .await?;
        let swap = self
            .build_swap_transaction(SwapRequest::new(quote.clone(), user_public_key))
            .await?;
        Ok((quote, swap))
    }

    pub fn estimate_out_amount_sol(&self, quote: &JupiterQuote, side: TradeSide) -> f64 {
        let raw = match side {
            TradeSide::Sell => &quote.out_amount,
            TradeSide::Buy => &quote.in_amount,
        };
        raw.parse::<u64>()
            .map(|v| v as f64 / 1_000_000_000.0)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn paper_quote_buy() {
        let client = JupiterClient::paper();
        let quote = client
            .get_quote(QuoteRequest::new(
                TradeSide::Buy,
                "PumpFunDemoMint1111111111111111111111111111",
                100_000_000,
            ))
            .await
            .unwrap();
        assert_eq!(quote.mode, JupiterMode::Paper);
        assert!(quote.out_amount.parse::<u64>().unwrap() > 0);
    }

    #[tokio::test]
    async fn paper_build_swap() {
        let client = JupiterClient::paper();
        let quote = client
            .get_quote(QuoteRequest::new(TradeSide::Buy, "mint", 50_000_000))
            .await
            .unwrap();
        let swap = client
            .build_swap_transaction(SwapRequest::new(quote, "wallet"))
            .await
            .unwrap();
        assert!(!swap.swap_transaction.is_empty());
    }
}
