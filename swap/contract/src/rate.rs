use crate::{
    constants::{MIN_SWAP_AMOUNT, RATE_TIERS},
    error::SwapError,
};
use casper_types::U512;

#[derive(Clone, Copy, Debug)]
pub struct RateTier {
    pub cspr_amount: U512,
    pub rate: U512,
}

pub fn validate_rate(rate: U512) -> Result<(), SwapError> {
    if rate.is_zero() {
        return Err(SwapError::InvalidRate);
    }
    Ok(())
}

pub fn validate_amount(amount: U512) -> Result<(), SwapError> {
    if amount == U512::zero() {
        return Err(SwapError::InvalidAmount);
    }
    Ok(())
}

/// Get the swap rate based on the CSPR amount.
pub fn get_swap_rate(cspr_amount: U512) -> Result<U512, SwapError> {
    if cspr_amount < MIN_SWAP_AMOUNT {
        return Err(SwapError::BelowMinimumSwap);
    }

    // Find the appropriate rate tier.
    let rate = RATE_TIERS
        .iter()
        .rev()
        .find(|tier| cspr_amount >= tier.cspr_amount)
        .map(|tier| tier.rate)
        .unwrap_or_else(|| RATE_TIERS.first().unwrap().rate); // Default to the base rate if no match.

    validate_rate(rate)?;
    Ok(rate)
}

#[cfg(feature = "contract-support")]
pub fn verify_swap_active() -> Result<(), SwapError> {
    use crate::constants::{ARG_END_TIME, ARG_START_TIME};
    use crate::utils::get_stored_value_with_user_errors;
    use casper_contract::contract_api::runtime::get_blocktime;

    // Get the current block time in milliseconds and convert to seconds
    let current_time_in_ms: u64 = get_blocktime().into();
    let current_time_in_seconds = current_time_in_ms / 1000;

    let start_time: u64 = get_stored_value_with_user_errors(
        ARG_START_TIME,
        SwapError::MissingStartTime,
        SwapError::InvalidStartTime,
    );
    let end_time: u64 = get_stored_value_with_user_errors(
        ARG_END_TIME,
        SwapError::MissingEndTime,
        SwapError::InvalidEndTime,
    );

    // Check if the current time falls within the swap window
    if current_time_in_seconds < start_time {
        return Err(SwapError::SwapNotActive);
    }
    if current_time_in_seconds > end_time {
        return Err(SwapError::SwapExpired);
    }

    Ok(())
}
