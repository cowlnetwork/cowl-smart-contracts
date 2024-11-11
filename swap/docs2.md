# First, store the wasm file path and network node
WASM_PATH="target/wasm32-unknown-unknown/release/cowl_ghost_swap.wasm"
NODE_ADDRESS="http://localhost:11101"
CHAIN_NAME="casper-test"

# Deploy the contract
casper-client put-deploy \
    --node-address $NODE_ADDRESS \
    --chain-name $CHAIN_NAME \
    --secret-key path/to/secret_key.pem \
    --payment-amount 150000000000 \
    --session-path $WASM_PATH \
    --session-arg "start_time:u64='$(date +%s)000'" \
    --session-arg "end_time:u64='$(date -d "+30 days" +%s)000'" \
    --session-arg "cowl_token:contract_hash='hash-COWL_CONTRACT_HASH_HERE'"


# Store contract hash for easy use
CONTRACT_HASH="hash-GHOST_SWAP_CONTRACT_HASH"

# Deposit 1 million COWL (adjust amount based on COWL token decimals)
casper-client put-deploy \
    --node-address $NODE_ADDRESS \
    --chain-name $CHAIN_NAME \
    --secret-key path/to/secret_key.pem \
    --payment-amount 5000000000 \
    --session-hash $CONTRACT_HASH \
    --session-entry-point "deposit_cowl" \
    --session-arg "amount:u512='1000000000000000000000000'" # Assuming 18 decimals

# First, store the user's purse URef
PURSE_UREF="uref-USER_PURSE_UREF_HERE"

# Swap 1000 CSPR for COWL
casper-client put-deploy \
    --node-address $NODE_ADDRESS \
    --chain-name $CHAIN_NAME \
    --secret-key path/to/user_key.pem \
    --payment-amount 5000000000 \
    --session-hash $CONTRACT_HASH \
    --session-entry-point "cspr_to_cowl" \
    --session-arg "amount:u512='1000000000000'" \
    --session-arg "recipient:key='account-RECIPIENT_ACCOUNT_HASH'" \
    --session-arg "purse:uref='$PURSE_UREF'"

# First approve COWL token spending (separate deploy to COWL token contract)
COWL_TOKEN_HASH="hash-COWL_CONTRACT_HASH"
casper-client put-deploy \
    --node-address $NODE_ADDRESS \
    --chain-name $CHAIN_NAME \
    --secret-key path/to/user_key.pem \
    --payment-amount 5000000000 \
    --session-hash $COWL_TOKEN_HASH \
    --session-entry-point "approve" \
    --session-arg "spender:key='$CONTRACT_HASH'" \
    --session-arg "amount:u256='3000000000000000000000'" # 3000 COWL with 18 decimals

# Then perform the swap
casper-client put-deploy \
    --node-address $NODE_ADDRESS \
    --chain-name $CHAIN_NAME \
    --secret-key path/to/user_key.pem \
    --payment-amount 5000000000 \
    --session-hash $CONTRACT_HASH \
    --session-entry-point "cowl_to_cspr" \
    --session-arg "amount:u512='3000000000000000000000'" \
    --session-arg "recipient:key='account-RECIPIENT_ACCOUNT_HASH'"

## helper
# Get contract hash after deployment
casper-client get-deploy --node-address $NODE_ADDRESS <DEPLOY_HASH>

# Get contract named keys
casper-client query-global-state \
    --node-address $NODE_ADDRESS \
    --key <CONTRACT_HASH> \
    --query-path ""

# Get user's main purse URef
casper-client get-account-info \
    --node-address $NODE_ADDRESS \
    --public-key path/to/public_key.pem

# Check COWL balance
casper-client query-global-state \
    --node-address $NODE_ADDRESS \
    --key <COWL_CONTRACT_HASH> \
    --query-path "balances/<ACCOUNT_HASH>"