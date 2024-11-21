# CEP-18 Token with Vesting - Deployment and Usage Guide

## Deployment Instructions

### 1. Prepare Deployment Arguments
```bash
# Required arguments for deployment
casper-client put-deploy \
    --node-address http://NODE_ADDRESS \
    --chain-name casper-test \
    --payment-amount PAYMENT_AMOUNT \
    --session-path path/to/contract.wasm \
    --session-arg "name:string='YourToken'" \
    --session-arg "symbol:string='TKN'" \
    --session-arg "decimals:u8='9'" \
    --session-arg "total_supply:u256='5500000000'" \  # Example: 5.5B tokens
    --session-arg "treasury_address:key='hash-0123...'" \
    --session-arg "team_address:key='hash-4567...'" \
    --session-arg "staking_address:key='hash-89ab...'" \
    --session-arg "investor_address:key='hash-cdef...'" \
    --session-arg "network_address:key='hash-0123...'" \
    --session-arg "marketing_address:key='hash-4567...'" \
    --session-arg "airdrop_address:key='hash-89ab...'" \
    --session-arg "events_mode:u8='1'" \
    --session-arg "enable_mint_burn:u8='0'"
```

### 2. Optional Admin Setup
```bash
# Add admin addresses during deployment
    --session-arg "admin_list:list<key>='[\"hash-0123...\", \"hash-4567...\"]'"

# Add minter addresses during deployment
    --session-arg "minter_list:list<key>='[\"hash-89ab...\", \"hash-cdef...\"]'"
```

## Checking Vesting Status

### 1. Treasury Reserve (50% of Supply)
```bash
# Query treasury vesting status
casper-client query-global-state \
    --node-address http://NODE_ADDRESS \
    --state-root-hash STATE_ROOT_HASH \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='treasury'"

# Response interpretation:
(
    total_amount: 2750000000,      # 50% of total supply
    vested_amount: 0,              # Amount available for transfer
    is_fully_vested: false         # True after 2 years
)
```

### 2. Team & Developers (7% of Supply)
```bash
# Query team vesting status
casper-client query-global-state \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='team'"

# Response interpretation:
(
    total_amount: 385000000,       # 7% of total supply
    vested_amount: 16041667,       # Amount currently available (~monthly release)
    is_fully_vested: false         # True after 24 months
)
```

### 3. Community Staking (20% of Supply)
```bash
# Query staking vesting status
casper-client query-global-state \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='staking'"

# Response interpretation:
(
    total_amount: 1100000000,      # 20% of total supply
    vested_amount: 9166667,        # Amount currently available
    is_fully_vested: false         # True after 10 years
)
```

### 4. Investor Allocation (10% of Supply)
```bash
# Query investor vesting status
casper-client query-global-state \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='investor'"

# Response interpretation:
(
    total_amount: 550000000,       # 10% of total supply
    vested_amount: 22916667,       # Amount currently available
    is_fully_vested: false         # True after 24 months
)
```

### 5. Network Rewards (5% of Supply)
```bash
# Query network rewards vesting status
casper-client query-global-state \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='network'"

# Response interpretation:
(
    total_amount: 275000000,       # 5% of total supply
    vested_amount: 11458333,       # Amount currently available
    is_fully_vested: false         # True after 24 months
)
```

### 6. Marketing & Exchanges (5% of Supply)
```bash
# Query marketing vesting status
casper-client query-global-state \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='marketing'"

# Response interpretation:
(
    total_amount: 275000000,       # 5% of total supply
    vested_amount: 11458333,       # Amount currently available
    is_fully_vested: false         # True after 24 months
)
```

### 7. Airdrops & Community (3% of Supply)
```bash
# Query airdrop vesting status
casper-client query-global-state \
    --key CONTRACT_HASH \
    --query "[\"vesting_details\"]" \
    --session-arg "vesting_type:string='airdrop'"

# Response interpretation:
(
    total_amount: 165000000,       # 3% of total supply
    vested_amount: 6875000,        # Amount currently available
    is_fully_vested: false         # True after 24 months
)
```

## Monthly Vesting Schedule Example

For allocations with 24-month linear vesting (Team, Investors, Network, Marketing, Airdrop):
```
Month 1: 4.17% of allocation unlocked
Month 2: 8.33% of allocation unlocked
Month 3: 12.50% of allocation unlocked
...
Month 23: 95.83% of allocation unlocked
Month 24: 100% of allocation unlocked
```

For Community Staking (120-month linear vesting):
```
Month 1: 0.83% of allocation unlocked
Month 2: 1.67% of allocation unlocked
Month 3: 2.50% of allocation unlocked
...
Month 119: 99.17% of allocation unlocked
Month 120: 100% of allocation unlocked
```

## Important Notes
1. Vesting starts at contract deployment
2. Vesting checks are automatic during transfers
3. Cannot transfer more than vested amount
4. Treasury has no partial vesting - full unlock after 2 years
5. All other allocations vest linearly
6. Monthly amounts are approximate due to blockchain time calculations
