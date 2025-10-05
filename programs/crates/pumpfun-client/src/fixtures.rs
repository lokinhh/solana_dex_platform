use crate::types::PumpToken;

pub fn mock_tokens() -> Vec<PumpToken> {
    vec![
        PumpToken {
            mint: "So11111111111111111111111111111111111111112".into(),
            symbol: "SOL".into(),
            name: "Wrapped SOL".into(),
            price_usd: 145.2,
            market_cap: 68_000_000_000.0,
            bonding_curve_pct: 100.0,
            holders: 0,
            source: "native".into(),
            volume_24h: None,
            created_at: None,
        },
        PumpToken {
            mint: "PumpFunDemoMint1111111111111111111111111111".into(),
            symbol: "PEPE2".into(),
            name: "Pepe 2.0".into(),
            price_usd: 0.000042,
            market_cap: 420_000.0,
            bonding_curve_pct: 78.0,
            holders: 1240,
            source: "pump.fun".into(),
            volume_24h: Some(250_000.0),
            created_at: Some(chrono::Utc::now().timestamp() - 3600),
        },
        PumpToken {
            mint: "PumpFunDemoMint2222222222222222222222222222".into(),
            symbol: "BONKAI".into(),
            name: "Bonk AI".into(),
            price_usd: 0.000018,
            market_cap: 180_000.0,
            bonding_curve_pct: 45.0,
            holders: 890,
            source: "pump.fun".into(),
            volume_24h: Some(95_000.0),
            created_at: Some(chrono::Utc::now().timestamp() - 7200),
        },
        PumpToken {
            mint: "PumpFunDemoMint3333333333333333333333333333".into(),
            symbol: "WIF2".into(),
            name: "Wif Sequel".into(),
            price_usd: 0.000095,
            market_cap: 950_000.0,
            bonding_curve_pct: 92.0,
            holders: 3100,
            source: "pump.fun".into(),
            volume_24h: Some(410_000.0),
            created_at: Some(chrono::Utc::now().timestamp() - 1800),
        },
        PumpToken {
            mint: "PumpFunDemoMint4444444444444444444444444444".into(),
            symbol: "MOON".into(),
            name: "Moon Shot".into(),
            price_usd: 0.00012,
            market_cap: 1_200_000.0,
            bonding_curve_pct: 88.0,
            holders: 4500,
            source: "pump.fun".into(),
            volume_24h: Some(520_000.0),
            created_at: Some(chrono::Utc::now().timestamp() - 900),
        },
        PumpToken {
            mint: "PumpFunDemoMint5555555555555555555555555555".into(),
            symbol: "DEGEN".into(),
            name: "Degen Alpha".into(),
            price_usd: 0.000008,
            market_cap: 80_000.0,
            bonding_curve_pct: 32.0,
            holders: 420,
            source: "pump.fun".into(),
            volume_24h: Some(35_000.0),
            created_at: Some(chrono::Utc::now().timestamp() - 14_400),
        },
    ]
}

pub fn enrich_paper_token(mut token: PumpToken, seed: u64) -> PumpToken {
    token.volume_24h = Some((seed % 500_000) as f64 + 10_000.0);
    token.created_at = Some(chrono::Utc::now().timestamp() - (seed % 86_400) as i64);
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_list_not_empty() {
        assert!(mock_tokens().len() >= 4);
    }
}
