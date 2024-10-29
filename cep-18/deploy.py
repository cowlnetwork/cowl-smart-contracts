#!/usr/bin/env python3

import argparse
import json
import os
import subprocess
import time
from datetime import datetime
from pathlib import Path

class COWLTokenDeployer:
    def __init__(self, network: str):
        self.network = network
        self.node_address = self._get_node_address()
        
    def _get_node_address(self):
        if self.network == "testnet":
            return "http://testnet.casper.network:7777"
        elif self.network == "mainnet":
            return "http://mainnet.casper.network:7777"
        return "http://localhost:11101"

    def deploy(self, secret_key_path: str, authorized_signers: list, 
               distribution_addresses: dict):
        # Prepare args
        deploy_args = [
            "casper-client",
            "put-deploy",
            "--node-address", self.node_address,
            "--chain-name", self.network,
            "--secret-key", secret_key_path,
            "--payment-amount", "150000000",
            "--session-path", 
            "target/wasm32-unknown-unknown/release/cowl_token.wasm",
            "--session-arg", 
            f"authorized_signers:vec[key]={json.dumps(authorized_signers)}",
            "--session-arg",
            f"distribution_addresses:vec[(string,key)]={json.dumps(distribution_addresses)}"
        ]

        # Execute deploy
        result = subprocess.run(deploy_args, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise Exception(f"Deploy failed: {result.stderr}")
            
        # Parse deploy hash
        deploy_hash = None
        for line in result.stdout.split('\n'):
            if "Deploy hash:" in line:
                deploy_hash = line.split(':')[1].strip()
                
        return deploy_hash

    def wait_for_deploy(self, deploy_hash: str, max_attempts: int = 100):
        print(f"Waiting for deploy {deploy_hash}...")
        
        for _ in range(max_attempts):
            result = subprocess.run([
                "casper-client",
                "get-deploy",
                deploy_hash,
                "--node-address",
                self.node_address
            ], capture_output=True, text=True)
            
            if "execution_results" in result.stdout:
                if "Success" in result.stdout:
                    print("Deploy successful!")
                    return True
                elif "Failure" in result.stdout:
                    print("Deploy failed!")
                    return False
                    
            time.sleep(2)
            
        print("Deploy timeout!")
        return False

def main():
    parser = argparse.ArgumentParser(description='Deploy COWL Token Contract')
    parser.add_argument('--network', choices=['testnet', 'mainnet', 'local'],
                      required=True)
    parser.add_argument('--secret-key', required=True,
                      help='Path to secret key PEM file')
    parser.add_argument('--config', required=True,
                      help='Path to deployment config JSON')
    
    args = parser.parse_args()
    
    # Load deployment config
    with open(args.config) as f:
        config = json.load(f)
    
    deployer = COWLTokenDeployer(args.network)
    
    # Deploy contract
    deploy_hash = deployer.deploy(
        args.secret_key,
        config['authorized_signers'],
        config['distribution_addresses']
    )
    
    # Wait for deployment
    if deployer.wait_for_deploy(deploy_hash):
        print("Contract deployed successfully!")
    else:
        print("Contract deployment failed!")

if __name__ == "__main__":
    main()
