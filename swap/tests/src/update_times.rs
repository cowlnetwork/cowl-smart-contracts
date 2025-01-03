use cowl_swap::error::SwapError;
use vesting_tests::support::assert_expected_error;

use crate::utility::installer_request_builders::{cowl_swap_update_times, setup, TestContext};

#[test]
fn should_update_times_contract() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            ..
        },
    ) = setup();

    let update_times =
        cowl_swap_update_times(&mut builder, &cowl_swap_contract_hash, 10_u64, 1000_u64);

    update_times.expect_success().commit();
}

#[test]
fn should_fail_update_times_contract() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            ..
        },
    ) = setup();

    let update_times =
        cowl_swap_update_times(&mut builder, &cowl_swap_contract_hash, 1000_u64, 10_u64);
    update_times.expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        SwapError::InvalidTimeWindow as u16,
        "InvalidTimeWindow",
    );
}
