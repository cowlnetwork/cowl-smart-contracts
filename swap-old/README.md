# COWL Ghost Swap Smart Contract

## Overview

The COWL Ghost Swap smart contract facilitates a controlled token sale environment between CSPR (Casper Network's native token) and COWL tokens (CEP-18 standard). The contract implements time-bound swapping with configured limits, rates, and tax mechanisms.

## Core Requirements

### Token Standard
- Works exclusively with COWL token implementing the CEP-18 standard
- Requires COWL token contract hash during deployment
- Upgradeable contract

### Contract Activation Parameters
1. **Time Window**
   - Configured at installation or upgrade
   - Default duration: 30 days
   - Parameters:
     - `start_time`: Unix timestamp for activation start
     - `end_time`: Unix timestamp for activation end

2. **Sale Limits**
   - Maximum sale amount per activation: 100 million COWL
   - Enforced across all swap operations
   - Parameters (optional):
     - `max_sale_amount`: Maximum sale amount per activation

3. **Tax Configuration**
   - Default tax rate: 10%
   - Applied only to COWL to CSPR conversions
   - Tax is retained in contract purse
   - Parameters (optional):
     - `tax_rate`: Percentage of sale amount taxed per swap operation

### Access Control

#### Owner Functions
1. **Deposit Operations**
   - Deposit CSPR to contract purse
   - Deposit COWL tokens to contract

2. **Withdrawal Operations**
   - Withdraw CSPR from contract purse
   - Withdraw COWL tokens from contract

#### User Functions
1. **CSPR to COWL Swap** (`cspr_to_cowl`)
   - No tax commission
   - Rate tiers based on CSPR amount:
     | CSPR Amount (motes) | Rate |
     |--------------------|------|
     | < 50,000           | 2x   |
     | ≥ 50,000          | 3x   |
     | ≥ 100,000         | 4x   |
     | ≥ 500,000         | 5x   |
     | ≥ 1,000,000       | 6x   |

2. **COWL to CSPR Swap** (`cowl_to_cspr`)
   - Includes 10% tax levy
   - Uses same rate tiers as CSPR to COWL
   - Final CSPR amount = (COWL Amount / Rate) * 0.9

### Example Transactions

**CSPR to COWL:**
```
Input: 60,000 CSPR
Rate: 3x
Output: 180,000 COWL
Tax: None
```

**COWL to CSPR:**
```
Input: 180,000 COWL
Rate: 3x
Base CSPR: 60,000 CSPR
Tax (10%): 6,000 CSPR
Final Output: 54,000 CSPR
```

## Contract Installation

### Required Parameters
```rust
runtime::get_named_arg("start_time"); // Unix timestamp
runtime::get_named_arg("end_time");   // Unix timestamp
runtime::get_named_arg("cowl_token"); // CEP-18 token hash
```

### Optional Parameters
```rust
runtime::get_named_arg("max_sale_amount"); // Maximum sale amount per activation
runtime::get_named_arg("tax_rate");        // Tax rate for COWL to CSPR swaps
```

### Installation Steps
1. Provide COWL token contract hash
2. Set activation time window
3. Configure owner account
4. Create contract purse
5. Deploy entry points

## Error States

- SwapNotActive (1): Outside valid time window
- SwapExpired (2): Past end time
- InsufficientBalance (3): Inadequate funds
- BelowMinimumSwap (4): Amount too small
- Unauthorized (5): Non-owner for restricted functions
- InvalidParameter (6): Bad input parameters
- TransferFailed (7): Token transfer issue
- InvalidRate (8): Rate calculation error
- ZeroAmount (9): Invalid zero amount
- InvalidTimeWindow (10): Bad time configuration

## Security Considerations

1. **Balance Management**
   - Contract maintains separate CSPR purse
   - Tracks COWL token balance
   - Validates all transfers

2. **Time Controls**
   - Strict enforcement of activation window
   - Owner-only time updates

3. **Access Control**
   - Owner-only administrative functions
   - Public swap functions with validations

4. **Rate Protection**
   - Fixed rate tiers
   - Minimum amount enforcement
   - Tax calculation safety

## Development Notes

1. **Dependencies**
   - Casper Network compatible
   - No standard library (`no_std`)
   - CEP-18 token interface

2. **Testing Requirements**
   - Time window validation
   - Rate tier calculations
   - Tax applications
   - Balance tracking
   - Access control
   - Error handling

3. **Monitoring Needs**
   - Sale amount tracking
   - Tax collection
   - Balance reconciliation
   - Time window status

## License

[License Information Here]

## Contributing

[Contribution Guidelines Here]