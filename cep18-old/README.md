# COWL Token Implementation (CEP-18 with Vesting)

## Overview
This is an implementation of the CEP-18 standard for fungible tokens on the Casper Network, extended with vesting and multi-signature capabilities.

## Token Specifications

### Base Configuration
- Name: "COWL Token"
- Symbol: "COWL"
- Decimals: 9 (Aligned with CSPR for consistent calculations)
- Base Unit: 1 COWL = 1,000,000,000 (1e9)
- Total Supply: 5,500,000,000 COWL (5.5e18 in base units)

### Distribution & Vesting Schedule

| Allocation  | % of Supply | COWL Amount    | Base Units (1e9)    | Duration | Vesting Type | Sign Authority |
|------------|-------------|----------------|---------------------|----------|--------------|----------------|
| Treasury   | 30%         | 1,650,000,000  | 1.65e18            | 2 years  | Full Lock    | Multi-sig      |
| Team       | 7%          | 385,000,000    | 3.85e17            | 2 years  | Linear       | Multi-sig      |
| Staking    | 20%         | 1,100,000,000  | 1.1e18             | 10 years | Linear       | Multi-sig      |
| Investor   | 10%         | 550,000,000    | 5.5e17             | 2 years  | Linear       | Multi-sig      |
| Network    | 5%          | 275,000,000    | 2.75e17            | 2 years  | Linear       | Multi-sig      |
| Marketing  | 5%          | 275,000,000    | 2.75e17            | 2 years  | Linear       | Single-sig     |
| Airdrop    | 3%          | 165,000,000    | 1.65e17            | 2 years  | Linear       | Single-sig     |
| Liquidity  | 20%         | 1,100,000,000  | 1.1e18             | None     | None         | Single-sig     |

## Contract Implementation

### Required CEP-18 Interfaces

```rust
// Basic Token Information
fn name() -> String;
fn symbol() -> String;
fn decimals() -> u8;
fn total_supply() -> U256;

// Balances & Allowances
fn balance_of(address: Key) -> U256;
fn allowance(owner: Key, spender: Key) -> U256;

// Transfer Methods
fn transfer(recipient: Key, amount: U256) -> Result<(), Error>;
fn transfer_from(owner: Key, recipient: Key, amount: U256) -> Result<(), Error>;

// Approval Methods
fn approve(spender: Key, amount: U256) -> Result<(), Error>;
fn increase_allowance(spender: Key, amount: U256) -> Result<(), Error>;
fn decrease_allowance(spender: Key, amount: U256) -> Result<(), Error>;
```

### Authorization Structure

```rust
// Contract Administration
pub struct ContractAccess {
    owner: Key,    // Primary contract administrator
    admin1: Key,   // Secondary administrator
    admin2: Key    // Tertiary administrator
}

// Multi-signature Configuration
pub struct MultiSigAccount {
    required_count: u32,
    signers: Vec<Key>,
    threshold_amount: U512
}
```

### Contract Entry Points

```rust
// Configuration & Admin
#[no_mangle]
pub extern "C" fn update_multi_sig_config() {
    let allocation_type: String = runtime::get_named_arg("allocation_type");
    let signers: Vec<Key> = runtime::get_named_arg("signers");
    let required_count: u32 = runtime::get_named_arg("required_count");
}

// Vesting Information
#[no_mangle]
pub extern "C" fn vesting_details() {
    let vesting_type: String = runtime::get_named_arg("vesting_type");
    // Returns (total_amount, vested_amount, is_fully_vested)
}
```

### Contract Installation

```rust
runtime_args! {
    // Token Configuration
    "name" => "COWL Token",
    "symbol" => "COWL",
    "decimals" => 9u8,
    "total_supply" => U256::from(5_500_000_000_000_000_000u64), // 5.5B with 9 decimals

    // Administration
    "contract_owner" => Key::Account(owner_account_hash),
    "admin1" => Key::Account(admin1_account_hash),
    "admin2" => Key::Account(admin2_account_hash),

    // Multi-sig Allocations
    "treasury_address" => Key::Hash(treasury_hash),
    "team_address" => Key::Hash(team_hash),
    "staking_address" => Key::Hash(staking_hash),
    "investor_address" => Key::Hash(investor_hash),
    "network_address" => Key::Hash(network_hash),

    // Single-sig Allocations
    "marketing_address" => Key::Account(marketing_account_hash),
    "airdrop_address" => Key::Account(airdrop_account_hash),
    "liquidity_address" => Key::Account(liquidity_account_hash)
}
```

### Error Handling

```rust
pub enum Error {
    // CEP-18 Standard Errors
    InsufficientBalance,
    InsufficientAllowance,
    Overflow,
    
    // Authorization Errors
    InsufficientSignatures,
    UnauthorizedSigner,
    
    // Vesting Errors
    VestingLocked,
    InvalidVestingType,
    
    // Parameter Errors
    InvalidParameter
}
```

## Development & Testing

### Prerequisites
- Rust with `wasm32-unknown-unknown` target
- Casper client tools
- Make utility

### Build Process
```bash
# Install prerequisites
make prepare

# Run tests
make test

# Build contract
make build-contract
```

### Contract Location
```
target/wasm32-unknown-unknown/release/cep18.wasm
```

### Testing with Casper Client
```bash
casper-client put-deploy \
    --node-address http://localhost:11101 \
    --chain-name casper-net-1 \
    --payment-amount 100000000000 \
    --session-path target/wasm32-unknown-unknown/release/cep18.wasm \
    --session-arg "name:string='COWL Token'" \
    --session-arg "symbol:string='COWL'" \
    --session-arg "decimals:u8='9'" \
    --session-arg "total_supply:u256='5500000000000000000'"
```

## Security Considerations

1. **Multi-signature Implementation**
   - Separate storage for each multi-sig configuration
   - Threshold validation
   - Signature verification

2. **Vesting Controls**
   - Time-based release calculations
   - Linear vesting enforcement
   - Lock period validation

3. **Administrative Security**
   - Three-tier admin structure
   - Role-based access control
   - Event logging for admin actions

## Development Resources

- [Casper Fungible Token Tutorial](/docs/full-tutorial.md)
- [CEP-18 Standard Documentation](/docs/cep-18-standard.md)
- [JavaScript SDK Documentation](https://github.com/casper-ecosystem/cep18/tree/master/client-js#readme)

## License

[License Information Here]

## Contributing

[Contribution Guidelines Here]