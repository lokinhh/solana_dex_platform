use crate::error::TradeRegistryError;

pub const MIN_SUBSCRIBE_BPS: u16 = 1;
pub const MAX_SUBSCRIBE_BPS: u16 = 10_000;
pub const MAX_FOLLOWERS_CAP: u32 = 10_000;

pub fn validate_subscribe_bps(size_bps: u16) -> Result<(), TradeRegistryError> {
    if size_bps < MIN_SUBSCRIBE_BPS || size_bps > MAX_SUBSCRIBE_BPS {
        return Err(TradeRegistryError::InvalidSizeBps);
    }
    Ok(())
}

pub fn validate_leader_cap(max_followers: u32) -> Result<(), TradeRegistryError> {
    if max_followers == 0 || max_followers > MAX_FOLLOWERS_CAP {
        return Err(TradeRegistryError::InvalidAccountData);
    }
    Ok(())
}

pub fn validate_lamports(amount: u64) -> Result<(), TradeRegistryError> {
    if amount == 0 {
        return Err(TradeRegistryError::InvalidAccountData);
    }
    Ok(())
}

pub fn scale_copy_amount_lamports(leader_lamports: u64, size_bps: u16) -> Result<u64, TradeRegistryError> {
    validate_subscribe_bps(size_bps)?;
    leader_lamports
        .checked_mul(size_bps as u64)
        .and_then(|v| v.checked_div(10_000))
        .ok_or(TradeRegistryError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_copy_size() {
        let scaled = scale_copy_amount_lamports(1_000_000_000, 5000).unwrap();
        assert_eq!(scaled, 500_000_000);
    }

    #[test]
    fn rejects_invalid_bps() {
        assert!(validate_subscribe_bps(0).is_err());
        assert!(validate_subscribe_bps(20_000).is_err());
    }
}
