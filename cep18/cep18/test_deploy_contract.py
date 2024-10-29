##
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
    def __init__(self, node_address: str, wasm_path: str):
        self.config = {
            "node_address": node_address,
            "chain_name": "casper-test",
            "payment_amount": "100000000000",  # 100 CSPR
            "wasm_path": wasm_path,
            
            # Token configuration
            "token_name": "CasperToken",
            "token_symbol": "CST",
            "decimals": 9,
            "total_supply": "5500000000000000000",  # 5.5B tokens with 9 decimals
            
            # Vesting percentages
            "allocations": {
                "treasury": 50,
                "team": 7,
                "staking": 20,
                "investor": 10,
                "network": 5,
                "marketing": 5,
                "airdrop": 3
            }
        }
        self.addresses = {}
        self.deploy_hash = None

        # Validate WASM file exists
        if not os.path.exists(wasm_path):
            print(f"Error: WASM file not found at {wasm_path}")
            sys.exit(1)

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
            "airdrop",
            "admin1",
            "admin2",
            "minter1",
            "minter2"
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
                # Keep node_address and wasm_path from initialization
                loaded_config = data["config"]
                loaded_config["node_address"] = self.config["node_address"]
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
            "--session-arg", f"name:string='{self.config['token_name']}'",
            "--session-arg", f"symbol:string='{self.config['token_symbol']}'",
            "--session-arg", f"decimals:u8='{self.config['decimals']}'",
            "--session-arg", f"total_supply:u256='{self.config['total_supply']}'",
            
            # Vesting addresses
            "--session-arg", f"treasury_address:key='{self.addresses['treasury']['public_key']}'",
            "--session-arg", f"team_address:key='{self.addresses['team']['public_key']}'",
            "--session-arg", f"staking_address:key='{self.addresses['staking']['public_key']}'",
            "--session-arg", f"investor_address:key='{self.addresses['investor']['public_key']}'",
            "--session-arg", f"network_address:key='{self.addresses['network']['public_key']}'",
            "--session-arg", f"marketing_address:key='{self.addresses['marketing']['public_key']}'",
            "--session-arg", f"airdrop_address:key='{self.addresses['airdrop']['public_key']}'",
            
            # Admin and minter lists
            "--session-arg", f"admin_list:list<key>='[\"{self.addresses['admin1']['public_key']}\", \"{self.addresses['admin2']['public_key']}\"]'",
            "--session-arg", f"minter_list:list<key>='[\"{self.addresses['minter1']['public_key']}\", \"{self.addresses['minter2']['public_key']}\"]'",
            
            # Other settings
            "--session-arg", "events_mode:u8='1'",
            "--session-arg", "enable_mint_burn:u8='0'",
            
            # Signing keys
            "--secret-key", f"./keys/deployer/secret_key.pem"
        ]
        
        return args

    def deploy_contract(self):
        """Deploy the contract"""
        try:
            print("\nPreparing deployment arguments...")
            args = self.prepare_deploy_args()
            
            print("\nDeploying contract...")
            result = subprocess.run(
                args, 
                capture_output=True, 
                text=True, 
                check=True
            )
            
            # Extract deploy hash from output
            self.deploy_hash = result.stdout.strip()
            print(f"Contract deployed successfully. Deploy hash: {self.deploy_hash}")
            
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

def parse_args():
    parser = argparse.ArgumentParser(description='Deploy CEP-18 Token Contract')
    parser.add_argument(
        '--node-address',
        required=True,
        help='Casper node address (e.g., http://localhost:11101)'
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
WASM Path: {args.wasm_path}
Clean Deploy: {args.clean}
""")
    
    # Initialize deployment
    deployment = TokenDeployment(args.node_address, args.wasm_path)
    
    # Load existing configuration unless clean flag is set
    if not args.clean and deployment.load_configuration():
        proceed = input("\nExisting configuration found. Proceed with deployment? (y/n): ")
        if proceed.lower() != 'y':
            print("Deployment cancelled")
            return
    else:
        # Create new addresses
        print("\nSetting up new addresses...")
        deployment.setup_addresses()
        deployment.save_configuration()
    
    # Final confirmation
    proceed = input("\nReady to deploy. Proceed? (y/n): ")
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
