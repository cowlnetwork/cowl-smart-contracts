#!/usr/bin/python3
#
### Basic usage
## python deploy_script.py --node-address http://localhost:11101 --wasm-path ./contract.wasm

# Start fresh (ignore existing config)
## python deploy_script.py --node-address http://localhost:11101 --wasm-path ./contract.wasm --clean

# Get help
## python deploy_script.py --help
##

import json
import subprocess
from pathlib import Path
from typing import Dict, List
from decimal import Decimal
import argparse
import sys
import os

class TokenDeployment:
    def __init__(self, node_address: str, chain_name: str, wasm_path: str):
        self.config = {
            "node_address": node_address,
            "chain_name": chain_name,  # Now passed as parameter
            "payment_amount": "100000000000",  # 100 CSPR
            "wasm_path": wasm_path,
            
            # Token configuration
            "token_name": "DDCasperToken",
            "token_symbol": "DSTT",
            "decimals": 9,
            "total_supply": "5500000000000000000",  # 5.5B tokens with 9 decimals
        }


    def create_keypair(self, name: str) -> Dict[str, str]:
        """Create a new keypair using casper-client"""
        try:
            # Create a directory for keys if it doesn't exist
            Path("./keys").mkdir(exist_ok=True)
            
            # Generate keypair
            subprocess.run([
                "casper-client", "keygen", f"./keys/{name}"
            ], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            
            # Read public key hex
            with open(f"./keys/{name}/public_key_hex") as f:
                public_key = f.read().strip()
            
            print(f"Created keypair for {name}: {public_key}")
            return {
                "public_key": public_key,
                "path": f"./keys/{name}"
            }
        except subprocess.CalledProcessError as e:
            print(f"Error creating keypair for {name}: {e.stderr.decode()}")
            sys.exit(1)
        except Exception as e:
            print(f"Unexpected error creating keypair for {name}: {e}")
            sys.exit(1)

    def setup_addresses(self):
        """Create all required addresses for deployment"""
        required_addresses = [
            "deployer",  # Address deploying the contract
            "treasury",
            "team",
            "staking",
            "investor",
            "network",
            "marketing",
            "airdrop"
        ]
            
        print("\nGenerating keypairs for all required addresses...")
        for name in required_addresses:
            self.addresses[name] = self.create_keypair(name)

    def save_configuration(self, filename: str = "deployment_config.json"):
        """Save the current configuration and addresses"""
        config = {
            "config": self.config,
            "addresses": self.addresses,
            "deploy_hash": self.deploy_hash
        }
        
        try:
            with open(filename, 'w') as f:
                json.dump(config, f, indent=2)
            print(f"\nConfiguration saved to {filename}")
        except Exception as e:
            print(f"Error saving configuration: {e}")

    def load_configuration(self, filename: str = "deployment_config.json"):
        """Load configuration from file"""
        try:
            with open(filename) as f:
                data = json.load(f)
                # Keep node_address, chain_name, and wasm_path from initialization
                loaded_config = data["config"]
                loaded_config["node_address"] = self.config["node_address"]
                loaded_config["chain_name"] = self.config["chain_name"]  # Keep the passed chain name
                loaded_config["wasm_path"] = self.config["wasm_path"]
                self.config = loaded_config
                self.addresses = data["addresses"]
                self.deploy_hash = data.get("deploy_hash")
            print("\nConfiguration loaded successfully")
            return True
        except FileNotFoundError:
            print("\nNo configuration file found, will create new configuration")
            return False
        except Exception as e:
            print(f"Error loading configuration: {e}")
            return False

    def prepare_deploy_args(self) -> List[str]:
        """Prepare command line arguments for deployment"""
        args = [
            "casper-client", "put-deploy",
            "--node-address", self.config["node_address"],
            "--chain-name", self.config["chain_name"],
            "--payment-amount", self.config["payment_amount"],
            "--session-path", self.config["wasm_path"],
            
            # Token basics
            "--session-arg", f"\"name:string='{self.config['token_name']}'\"",
            "--session-arg", f"\"symbol:string='{self.config['token_symbol']}'\"",
            "--session-arg", f"\"decimals:u8='{self.config['decimals']}'\"",
            "--session-arg", f"\"total_supply:u256='{self.config['total_supply']}'\"",
            
            # Vesting addresses
            "--session-arg", f"\"treasury_address:key='{self.get_account_key(self.addresses['treasury']['public_key'])}'\"",
            "--session-arg", f"\"team_address:key='{self.get_account_key(self.addresses['team']['public_key'])}'\"",
            "--session-arg", f"\"staking_address:key='{self.get_account_key(self.addresses['staking']['public_key'])}'\"",
            "--session-arg", f"\"investor_address:key='{self.get_account_key(self.addresses['investor']['public_key'])}'\"",
            "--session-arg", f"\"network_address:key='{self.get_account_key(self.addresses['network']['public_key'])}'\"",
            "--session-arg", f"\"marketing_address:key='{self.get_account_key(self.addresses['marketing']['public_key'])}'\"",
            "--session-arg", f"\"airdrop_address:key='{self.get_account_key(self.addresses['airdrop']['public_key'])}'\"",
            
            # Signing keys
            "--secret-key", f"./keys/deployer/secret_key.pem"
        ]
        return args

    def deploy_contract(self):
        """Deploy the contract"""
        try:
            print("\nPreparing deployment arguments...")
            args = self.prepare_deploy_args()
            
            # Print deployment command
            self.print_deploy_command(args)
            
            print("\nDeploying contract...")
            result = subprocess.run(
                args,
                capture_output=True,
                text=True,
                check=True,
                shell=True
            )
            
            # Print deployment response
            self.print_deploy_response(result.stdout)
            self.print_deploy_response(result.stderr)
            
            # Extract deploy hash from output
            self.deploy_hash = result.stdout.strip()
            print(f"\nDeploy hash: {self.deploy_hash}")
            
            # Save updated configuration
            self.save_configuration()
            
        except subprocess.CalledProcessError as e:
            print(f"Error deploying contract: {e.stderr}")
            sys.exit(1)
        except Exception as e:
            print(f"Unexpected error during deployment: {e}")
            sys.exit(1)

    def verify_deployment(self):
        """Verify the deployment was successful"""
        if not self.deploy_hash:
            print("No deploy hash found")
            return False
        
        try:
            print("\nVerifying deployment...")
            result = subprocess.run([
                "casper-client", "get-deploy",
                self.deploy_hash,
                "--node-address", self.config["node_address"]
            ], capture_output=True, text=True, check=True)
            
            if "Success" in result.stdout:
                print("✅ Deployment verified successfully")
                return True
            else:
                print("❌ Deployment verification failed")
                return False
                
        except subprocess.CalledProcessError as e:
            print(f"Error verifying deployment: {e.stderr}")
            return False
        
    def print_deploy_command(self, args: List[str]):
        """Format and print the deployment command"""
        print("\nDeployment Command:")
        print("------------------")
        command = " ".join(args)
        print(command)

    def print_deploy_response(self, response: str):
        """Format and print the deployment response"""
        print("\nDeployment Response:")
        print("-------------------")
        try:
            # Try to parse as JSON for better formatting
            response_data = json.loads(response)
            print(json.dumps(response_data, indent=2))
        except json.JSONDecodeError:
            # If not JSON, print as is
            print(response)

    def get_account_key(self, public_key: str) -> str:
        """Convert public key to proper Key::Account format"""
        try:
            result = subprocess.run([
                "casper-client", "account-address",
                "--public-key", public_key
            ], capture_output=True, text=True, check=True)
            account_hash = result.stdout.strip()
            return f"{account_hash}"
        except subprocess.CalledProcessError as e:
            print(f"Error getting account hash: {e.stderr}")
            sys.exit(1)

def parse_args():
    parser = argparse.ArgumentParser(description='Deploy CEP-18 Token Contract')
    parser.add_argument(
        '--node-address',
        required=True,
        help='Casper node address (e.g., http://localhost:11101)'
    )
    parser.add_argument(
        '--chain-name',
        required=True,
        help='Chain name (e.g., casper-test)'
    )
    parser.add_argument(
        '--wasm-path',
        required=True,
        help='Path to the contract WASM file'
    )
    parser.add_argument(
        '--clean',
        action='store_true',
        help='Start with fresh configuration (ignore existing config)'
    )
    return parser.parse_args()

def main():
    args = parse_args()
    
    print(f"""
CEP-18 Token Deployment
----------------------
Node Address: {args.node_address}
Chain Name: {args.chain_name}
WASM Path: {args.wasm_path}
Clean Deploy: {args.clean}
""")
    
    # Initialize deployment
    deployment = TokenDeployment(args.node_address, args.chain_name, args.wasm_path)
    
    # Load existing configuration unless clean flag is set
    if not args.clean and deployment.load_configuration():
        proceed = 'y'
        #proceed = input("\nExisting configuration found. Proceed with deployment? (y/n): ")
        if proceed.lower() != 'y':
            print("Deployment cancelled")
            return
    else:
        # Create new addresses
        print("\nSetting up new addresses...")
        deployment.setup_addresses()
        deployment.save_configuration()
    
    # Final confirmation
    # proceed = input("\nReady to deploy. Proceed? (y/n): ")
    proceed ='y'
    if proceed.lower() != 'y':
        print("Deployment cancelled")
        return
    
    # Deploy contract
    deployment.deploy_contract()
    
    # Verify deployment
    if deployment.verify_deployment():
        print("\n🎉 Deployment complete and verified!")
        print(f"Configuration saved to: {os.path.abspath('deployment_config.json')}")
    else:
        print("\n❌ Deployment verification failed")
        sys.exit(1)

if __name__ == "__main__":
    main()
