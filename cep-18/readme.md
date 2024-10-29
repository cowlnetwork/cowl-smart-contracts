# Run tests
make test

# Test specific functionality
cargo test test_vesting_schedule

## Deploy
casper-client put-deploy \
  --node-address http://NODE_ADDRESS \
  --chain-name casper-test \
  --secret-key path/to/secret_key.pem \
  --payment-amount 100000000 \
  --session-path target/wasm32-unknown-unknown/release/cowl_token.wasm \
  --session-arg "authorized_signers:vec[key]=['key1', 'key2', 'key3', 'key4', 'key5']" \
  --session-arg "distribution_addresses:vec[(string,key)]=[('treasury', 'addr1'), ...]"



## Change Address
# First signer
casper-client put-deploy \
  --session-hash <contract-hash> \
  --session-entry-point "propose_address_change" \
  --session-arg "category:string='team'" \
  --session-arg "new_address:key='new-address'"

# Second signer
casper-client put-deploy \
  --session-hash <contract-hash> \
  --session-entry-point "sign_proposal" \
  --session-arg "proposal_id:string='team_timestamp'"



python3 scripts/generate_addresses.py

python3 scripts/generate_addresses.py --output-dir /path/to/keys

python3 scripts/generate_addresses.py --verify
