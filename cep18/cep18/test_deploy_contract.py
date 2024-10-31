#!/usr/bin/python3
import json
import subprocess
from pathlib import Path
from typing import Dict, List
from decimal import Decimal
import argparse
import sys
import os

print("Starting script...")  # Debug print

class TokenDeployment:
    def __init__(self, node_address: str, chain_name: str, wasm_path: str, token_name: str):
        print(f"Initializing TokenDeployment with {node_address}, {chain_name}, {wasm_path}, {token_name}")  # Debug print
        self.node_address = node_address
        self.chain_name = chain_name
        self.wasm_path = wasm_path
        
        # Initialize addresses dictionary
        self.addresses = {}
        self.deploy_hash = None
        
        # Token configuration
        self.config = {
            "token_name": token_name,
            "token_symbol": token_name.upper(),  # Convert to uppercase for symbol
            "decimals": 9,
            "total_supply": "5500000000000000000",  # 5.5B tokens with 9 decimals
            "payment_amount": "300000000000",  # 100 CSPR
        }
        print("Initialization complete")  # Debug print

    def create_keypair(self, name: str) -> Dict[str, str]:
        """Create a new keypair using casper-client"""
        print(f"Creating keypair for {name}")  # Debug print
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
        print("Setting up addresses...")  # Debug print
        required_addresses = [
            "deployer",  # Address deploying the contract
            "treasury",
            "team",
            "staking",
            "investor",
            "network",
            "marketing",
            "airdrop",
            "liquidity"  # Added liquidity address
        ]
            
        print("\nGenerating keypairs for all required addresses...")
        for name in required_addresses:
            self.addresses[name] = self.create_keypair(name)
        print("Address setup complete")  # Debug print

    def load_configuration(self, filename: str = "deployment_config.json"):
        """Load configuration from file"""
        print("Loading configuration...")  # Debug print
        try:
            with open(filename) as f:
                data = json.load(f)
                self.config.update(data["config"])
                self.addresses = data["addresses"]
                self.deploy_hash = data.get("deploy_hash")
            print("Configuration loaded successfully")
            return True
        except FileNotFoundError:
            print("No configuration file found, will create new configuration")
            self.setup_addresses()  # Create addresses if file doesn't exist
            return False
        except Exception as e:
            print(f"Error loading configuration: {e}")
            return False

    def save_configuration(self, filename: str = "deployment_config.json"):
        """Save the current configuration and addresses"""
        print("Saving configuration...")  # Debug print
        config = {
            "config": self.config,
            "addresses": self.addresses,
            "deploy_hash": self.deploy_hash
        }
        
        try:
            with open(filename, 'w') as f:
                json.dump(config, f, indent=2)
            print(f"Configuration saved to {filename}")
        except Exception as e:
            print(f"Error saving configuration: {e}")

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

    def prepare_deploy_args(self) -> List[str]:
        """Prepare command line arguments for deployment"""
        print("Preparing deployment arguments...")  # Debug print
        args = [
            "casper-client", "put-deploy",
            "--node-address", self.node_address,
            "--chain-name", self.chain_name,
            "--payment-amount", self.config["payment_amount"],
            "--session-path", self.wasm_path,
            
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
            "--session-arg", f"\"liquidity_address:key='{self.get_account_key(self.addresses['liquidity']['public_key'])}'\"",
            
            # Signing keys
            "--secret-key", f"./keys/deployer/secret_key.pem"
        ]
        return args

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
                "--node-address", self.node_address
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

    def deploy_contract(self):
        """Deploy the contract"""
        try:
            print("\nPreparing deployment arguments...")
            args = self.prepare_deploy_args()
            
            # Print deployment command
            self.print_deploy_command(args)
            
            print("\nDeploying contract...")
            result = subprocess.run(
                " ".join(args),  # Join args with spaces for shell execution
                capture_output=True,
                text=True,
                check=False,  # Don't raise exception on non-zero exit
                shell=True
            )
            
            # Print full output for debugging
            print("STDOUT:", result.stdout)
            print("STDERR:", result.stderr)
            
            if result.returncode != 0:
                print(f"Deployment failed with return code: {result.returncode}")
                print(f"Error: {result.stderr}")
                sys.exit(1)
                
            # Extract deploy hash from output
            self.deploy_hash = result.stdout.strip()
            print(f"\nDeploy hash: {self.deploy_hash}")
            
            # Save updated configuration
            self.save_configuration()
            
        except Exception as e:
            print(f"Unexpected error during deployment: {str(e)}")
            import traceback
            traceback.print_exc()
            sys.exit(1)

def parse_args():
    print("Parsing arguments...")  # Debug print
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
        '--token-name',
        required=True,
        help='Name of the token (will be used for symbol in uppercase)'
    )
    parser.add_argument(
        '--clean',
        action='store_true',
        help='Start with fresh configuration (ignore existing config)'
    )
    args = parser.parse_args()
    print(f"Arguments parsed: {args}")  # Debug print
    return args

if __name__ == "__main__":
    try:
        print("Script started")  # Debug print
        args = parse_args()
        
        print(f"""
    CEP-18 Token Deployment
    ----------------------
    Node Address: {args.node_address}
    Chain Name: {args.chain_name}
    WASM Path: {args.wasm_path}
    Token Name: {args.token_name}
    Clean Deploy: {args.clean}
    """)
        
        # Initialize deployment
        deployment = TokenDeployment(args.node_address, args.chain_name, args.wasm_path, args.token_name)
        
        # Load existing configuration or create new if clean flag is set
        if args.clean or not deployment.load_configuration():
            # deployment.setup_addresses()
            deployment.save_configuration()
        
        # Deploy contract
        deployment.deploy_contract()
        
        # Verify deployment
        if deployment.verify_deployment():
            print("\n🎉 Deployment complete and verified!")
            print(f"Configuration saved to: {os.path.abspath('deployment_config.json')}")
        else:
            print("\n❌ Deployment verification failed")
            sys.exit(1)
    except Exception as e:
        print(f"Error in main: {e}")  # Debug print
        raise