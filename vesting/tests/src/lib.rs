#[cfg(test)]
mod install;

#[cfg(test)]
mod utility;

#[cfg(test)]
mod upgrade;

#[cfg(test)]
mod security;

#[cfg(test)]
mod modalities;

#[cfg(test)]
mod vesting_status;

#[cfg(test)]
mod vesting_info;

#[cfg(test)]
mod filter;

#[cfg(test)]
mod tests {
    use cowl_vesting::enums::VESTING_PERCENTAGES;
    #[test]
    fn test_vesting_percentages_sum_to_100() {
        let total_percentage: u8 = VESTING_PERCENTAGES
            .iter()
            .map(|&(_, percentage)| percentage)
            .sum();
        assert_eq!(
            total_percentage, 100,
            "VESTING_PERCENTAGES does not sum to 100%, actual total: {}%",
            total_percentage
        );
    }
}
