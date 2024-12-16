#[cfg(test)]
mod tests {
    use super::*;
    use casper_engine_test_support::{Code, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{account::AccountHash, runtime_args, RuntimeArgs, U512};

    const CONTRACT_WASM: &str = "contract.wasm";
    const TOKEN_WASM: &str = "erc20_token.wasm";

    fn deploy_contract(
        context: &mut TestContext,
        sender: AccountHash,
        start_time: U256,
        end_time: U256,
        cowl_token: ContractHash,
    ) -> ContractHash {
        let session_code = Code::from(CONTRACT_WASM);
        let session_args = runtime_args! {
            "start_time" => start_time,
            "end_time" => end_time,
            "cowl_token" => cowl_token
        };

        let session = SessionBuilder::new(session_code, session_args)
            .with_address(sender)
            .with_authorization_keys(&[sender])
            .build();

        context.run(session);
        let contract_hash = context
            .query(None, &[CONTRACT_NAME.to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        contract_hash
    }

    #[test]
    fn test_swap_initialization() {
        let mut context = TestContextBuilder::new()
            .with_public_key("test_account", U512::from(500_000_000_000_000u64))
            .build();

        // Deploy COWL token first
        let token_code = Code::from(TOKEN_WASM);
        let token_args = runtime_args! {
            "name" => "COWL Token",
            "symbol" => "COWL",
            "decimals" => 9u8,
            "total_supply" => U256::from(1_000_000_000u64)
        };

        let session = SessionBuilder::new(token_code, token_args)
            .with_address(context.get_account("test_account").unwrap())
            .with_authorization_keys(&[context.get_account("test_account").unwrap()])
            .build();

        context.run(session);
        let token_hash: ContractHash = context
            .query(None, &["erc20_token_contract".to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        // Deploy swap contract
        let start_time = U256::from(runtime::get_blocktime());
        let end_time = start_time + U256::from(86400); // 24 hours from now

        let contract_hash = deploy_contract(
            &mut context,
            context.get_account("test_account").unwrap(),
            start_time,
            end_time,
            token_hash,
        );

        // Verify contract installation
        assert!(contract_hash.as_bytes().len() > 0);

        // Verify named keys
        let owner: AccountHash = context
            .query(None, &[CONTRACT_NAME.to_string(), named_keys::OWNER.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(owner, context.get_account("test_account").unwrap());

        let stored_start_time: U256 = context
            .query(None, &[CONTRACT_NAME.to_string(), named_keys::START_TIME.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(stored_start_time, start_time);

        let stored_end_time: U256 = context
            .query(None, &[CONTRACT_NAME.to_string(), named_keys::END_TIME.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(stored_end_time, end_time);
    }

    #[test]
    fn test_cspr_to_cowl_swap() {
        // Test implementation for CSPR to COWL swap
    }

    #[test]
    fn test_cowl_to_cspr_swap() {
        // Test implementation for COWL to CSPR swap
    }

    #[test]
    fn test_update_times() {
        // Test implementation for updating times
    }

    #[test]
    fn test_withdraw_functions() {
        // Test implementation for withdraw functions
    }
}