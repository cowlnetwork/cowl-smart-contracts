#!/usr/bin/python3

import json
import subprocess
import argparse
import sys

class FundDeployer:
    def __init__(self, node_address: str, chain_name: str, faucet_key: str):
        self.node_address = node_address
        self.chain_name = chain_name
        self.faucet_key = faucet_key
        self.config = None

    def load_deployment_config(self):
        """Load the deployment configuration to get deployer address"""
        try:
            with open("deployment_config.json", 'r') as f:
                self.config = json.load(f)
                return True
        except FileNotFoundError:
            print("Error: deployment_config.json not found")
            return False

    def transfer_funds(self, amount: str):
        """Transfer funds from faucet to deployer"""
        try:
            deployer_public_key = self.config["addresses"]["deployer"]["public_key"]
            
            print(f"Transferring {amount} motes to: {deployer_public_key}")
            
            transfer_args = [
                "casper-client", "transfer",
                "--node-address", self.node_address,
                "--chain-name", self.chain_name,
                "--transfer-id", "01",
                "--amount", amount,
                "--target-account", deployer_public_key,
                "--payment-amount", "100000000",
                "--secret-key", self.faucet_key
            ]
            
            result = subprocess.run(transfer_args, capture_output=True, text=True, check=True)
            deploy_hash = result.stdout.strip()
            print(f"Transfer deploy hash: {deploy_hash}")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"Transfer failed: {e.stderr}")
            return False

def main():
    parser = argparse.ArgumentParser(description='Fund deployer account')
    parser.add_argument('--node-address', required=True, help='Casper node address')
    parser.add_argument('--chain-name', required=True, help='Chain name')
    parser.add_argument('--amount', required=True, help='Amount in motes')
    parser.add_argument('--faucet-key', required=True, help='Path to faucet secret key')
    args = parser.parse_args()  # Fixed: Changed parse_args() to parser.parse_args()
    
    funder = FundDeployer(args.node_address, args.chain_name, args.faucet_key)
    
    if not funder.load_deployment_config():
        sys.exit(1)
    
    if funder.transfer_funds(args.amount):
        print("Transfer completed")
    else:
        print("Transfer failed")
        sys.exit(1)

if __name__ == "__main__":
    main()
