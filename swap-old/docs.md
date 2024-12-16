# First, let's define some common variables we'll use
CHAIN_NAME="casper-test"  # or "casper" for mainnet
NODE_ADDRESS="http://localhost:7777"  # or your preferred node
CONTRACT_HASH="hash-abc...123"        # your deployed contract hash
SECRET_KEY_PATH="path/to/secret_key.pem"
PAYMENT_AMOUNT=10000000000            # 10 CSPR for gas

# 1. CSPR to COWL Swap
# For swapping 50,000 CSPR to COWL tokens
casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "$NODE_ADDRESS" \
    --secret-key "$SECRET_KEY_PATH" \
    --payment-amount "$PAYMENT_AMOUNT" \
    --session-hash "$CONTRACT_HASH" \
    --session-entry-point "cspr_to_cowl" \
    --session-arg "amount:U512='50000000000000'" \
    --session-arg "recipient:key='account-hash-abc...123'"

# 2. COWL to CSPR Swap
# First, approve the contract to spend your COWL tokens
casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "$NODE_ADDRESS" \
    --secret-key "$SECRET_KEY_PATH" \
    --payment-amount "$PAYMENT_AMOUNT" \
    --session-hash "$COWL_TOKEN_HASH" \
    --session-entry-point "approve" \
    --session-arg "spender:key='contract-hash-abc...123'" \
    --session-arg "amount:U512='150000000000000'"  # Amount of COWL tokens (50,000 * 3 rate)

# Then perform the swap
casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "$NODE_ADDRESS" \
    --secret-key "$SECRET_KEY_PATH" \
    --payment-amount "$PAYMENT_AMOUNT" \
    --session-hash "$CONTRACT_HASH" \
    --session-entry-point "cowl_to_cspr" \
    --session-arg "amount:U512='150000000000000'" \
    --session-arg "recipient:key='account-hash-abc...123'"

# 3. Query contract state (useful for checking balances, times, etc.)
casper-client query-global-state \
    --node-address "$NODE_ADDRESS" \
    --state-root-hash "state-root-hash" \
    --key "$CONTRACT_HASH" \
    --query-path "start_time"

# 4. Check deploy status
casper-client get-deploy \
    --node-address "$NODE_ADDRESS" \
    --deploy-hash "deploy-hash-abc...123"

# Helper script to convert CSPR to motes
#!/bin/bash
convert_cspr_to_motes() {
    local cspr=$1
    local motes=$(echo "$cspr * 1000000000" | bc)
    echo $motes
}

# Helper script to check minimum swap amount
#!/bin/bash
check_minimum_swap() {
    local amount=$1
    local min_amount=50000000000000  # 50,000 CSPR in motes
    
    if [ "$amount" -lt "$min_amount" ]; then
        echo "Error: Amount below minimum swap requirement of 50,000 CSPR"
        exit 1
    fi
}

# Example usage script
#!/bin/bash
perform_cspr_to_cowl_swap() {
    local cspr_amount=$1
    local recipient_key=$2
    
    # Convert CSPR to motes
    local motes=$(convert_cspr_to_motes $cspr_amount)
    
    # Check minimum amount
    check_minimum_swap $motes
    
    # Perform swap
    local deploy_hash=$(casper-client put-deploy \
        --chain-name "$CHAIN_NAME" \
        --node-address "$NODE_ADDRESS" \
        --secret-key "$SECRET_KEY_PATH" \
        --payment-amount "$PAYMENT_AMOUNT" \
        --session-hash "$CONTRACT_HASH" \
        --session-entry-point "cspr_to_cowl" \
        --session-arg "amount:U512='$motes'" \
        --session-arg "recipient:key='$recipient_key'" \
        | grep "Deploy hash" | cut -d ' ' -f 3)
    
    echo "Swap initiated with deploy hash: $deploy_hash"
    
    # Wait for confirmation
    while true; do
        status=$(casper-client get-deploy \
            --node-address "$NODE_ADDRESS" \
            --deploy-hash "$deploy_hash" \
            | grep "execution_results" | grep "Success")
        
        if [ ! -z "$status" ]; then
            echo "Swap completed successfully"
            break
        fi
        
        echo "Waiting for confirmation..."
        sleep 10
    done
}

# Example of checking rates based on amount
#!/bin/bash
get_swap_rate() {
    local cspr_amount=$1
    
    if [ "$cspr_amount" -ge 1000000000000000 ]; then  # 1,000,000 CSPR
        echo 6
    elif [ "$cspr_amount" -ge 500000000000000 ]; then # 500,000 CSPR
        echo 5
    elif [ "$cspr_amount" -ge 100000000000000 ]; then # 100,000 CSPR
        echo 4
    elif [ "$cspr_amount" -ge 50000000000000 ]; then  # 50,000 CSPR
        echo 3
    else
        echo "Amount below minimum requirement"
        exit 1
    fi
}


# 1. First, get the contract package hash after deployment
CONTRACT_PACKAGE_HASH=$(casper-client query-global-state \
    --node-address http://localhost:7777 \
    --state-root-hash <STATE_ROOT_HASH> \
    --key <CONTRACT_HASH> \
    --query-path "self_package_hash" \
    | jq -r '.result.stored_value.CLValue.bytes')

# 2. Approve the contract to spend owner's COWL tokens
casper-client put-deploy \
    --chain-name casper-test \
    --node-address http://localhost:7777 \
    --secret-key /path/to/secret_key.pem \
    --payment-amount 5000000000 \
    --session-hash <COWL_TOKEN_HASH> \
    --session-entry-point "approve" \
    --session-arg "spender:key='hash-$CONTRACT_PACKAGE_HASH'" \
    --session-arg "amount:U512='1000000000000000'"

# 3. Initialize contract with COWL tokens
casper-client put-deploy \
    --chain-name casper-test \
    --node-address http://localhost:7777 \
    --secret-key /path/to/secret_key.pem \
    --payment-amount 5000000000 \
    --session-hash <CONTRACT_HASH> \
    --session-entry-point "init_cowl_balance" \
    --session-arg "amount:U512='1000000000000000'"

# 4. Verify contract's COWL balance
casper-client query-global-state \
    --node-address http://localhost:7777 \
    --state-root-hash <STATE_ROOT_HASH> \
    --key <COWL_TOKEN_HASH> \
    --query-path "balances/$CONTRACT_PACKAGE_HASH"
