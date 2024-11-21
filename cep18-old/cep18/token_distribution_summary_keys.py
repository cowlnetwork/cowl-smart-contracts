#!/usr/bin/python3
from pycspr.types import CL_Key
from pycspr import NodeClient
from pycspr import NodeConnection
from pycspr.types import DictionaryID_ContractNamedKey
import base64
import argparse
import sys
import json
import subprocess
from typing import Dict, Optional

def parse_args():
    parser = argparse.ArgumentParser(description='Get Token Balances from deployment config')
    parser.add_argument(
        '--node-url',
        default='localhost',
        help='Casper node URL (default: localhost)'
    )
    parser.add_argument(
        '--contract-key',
        required=True,
        help='Contract key hash'
    )
    parser.add_argument(
        '--config-file',
        default='deployment_config.json',
        help='Path to deployment config file (default: deployment_config.json)'
    )
    return parser.parse_args()

def load_addresses_from_config(config_file: str) -> Dict[str, str]:
    """Load addresses and their roles from deployment config file"""
    try:
        with open(config_file) as f:
            config = json.load(f)
            if "addresses" not in config:
                raise ValueError("No addresses found in config file")
            
            # Extract addresses with their roles
            addresses = {
                role: details["public_key"]
                for role, details in config["addresses"].items()
                if "public_key" in details
            }
            
            if not addresses:
                raise ValueError("No valid addresses found in config file")
                
            return addresses
            
    except FileNotFoundError:
        print(f"Error: Config file {config_file} not found")
        sys.exit(1)
    except json.JSONDecodeError:
        print(f"Error: Config file {config_file} is not valid JSON")
        sys.exit(1)
    except Exception as e:
        print(f"Error loading addresses from config: {str(e)}")
        sys.exit(1)

def get_account_hash(public_key: str) -> Optional[str]:
    """Convert public key to account hash using casper-client"""
    try:
        result = subprocess.run(
            ["casper-client", "account-address", "--public-key", public_key],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Warning: Could not get account hash for {public_key}: {e.stderr}")
        return None
    except Exception as e:
        print(f"Warning: Error processing {public_key}: {str(e)}")
        return None

def get_balance(client: NodeClient, account_hash: str, contract_key: str) -> Optional[str]:
    """Get balance for a single account"""
    try:
        # If the input is already an account hash, use it directly
        # Otherwise, assume it's a public key and convert it
        if account_hash.startswith('account-hash-'):
            account_key = CL_Key.from_string(account_hash)
        else:
            account_hash = get_account_hash(account_hash)
            if not account_hash:
                return None
            account_key = CL_Key.from_string(account_hash)

        # Create dictionary item key
        item_key = base64.b64encode(b'\x00' + account_key.identifier)

        # Create dictionary identifier
        dictionary_id = DictionaryID_ContractNamedKey(
            dictionary_name="balances",
            dictionary_item_key=item_key.decode("utf-8"),
            contract_key=contract_key
        )

        # Get dictionary item
        response = client.get_dictionary_item(dictionary_id)
        
        # Extract balance
        return response["stored_value"]["CLValue"]["parsed"]

    except Exception as e:
        print(f"Warning: Could not get balance for {account_hash}: {str(e)}")
        return None

def get_all_balances(node_url: str, contract_key: str, address_map: Dict[str, str]) -> Dict[str, str]:
    """Get balances for all addresses"""
    results = {}
    total_balance = 0
    
    try:
        # Create node client (only once for all queries)
        client = NodeClient(NodeConnection(
            host=node_url,
            port_rpc=7777
        ))

        for role, address in address_map.items():
            balance = get_balance(client, address, contract_key)
            if balance is not None:
                results[role] = {
                    "address": address,
                    "balance": balance
                }
                try:
                    total_balance += int(balance)
                except (ValueError, TypeError):
                    print(f"Warning: Could not add balance for {role} ({address}) to total")
            else:
                results[role] = {
                    "address": address,
                    "balance": "Address not found or balance unavailable"
                }
                
        return results, total_balance

    except Exception as e:
        print(f"Error connecting to node at {node_url}: {str(e)}")
        sys.exit(1)

    except Exception as e:
        print(f"Error connecting to node at {node_url}: {str(e)}")
        sys.exit(1)
def format_large_number(number: int) -> str:
    """Convert large numbers to readable format with B/M suffix"""
    try:
        number = float(number)
        if number >= 1_000_000_000:  # Billion
            return f"{number/1_000_000_000:.2f}B"
        elif number >= 1_000_000:     # Million
            return f"{number/1_000_000:.2f}M"
        else:
            return f"{number:,.0f}"
    except (ValueError, TypeError):
        return "Invalid number"

def main():
    try:
        # Parse command line arguments
        args = parse_args()
        
        # Load addresses from config file
        address_map = load_addresses_from_config(args.config_file)
        
        print(f"""
Token Balance Query (Deployment Addresses)
----------------------------------------
Node URL: {args.node_url}
Contract Key: {args.contract_key}
Config File: {args.config_file}
Number of addresses to check: {len(address_map)}
""")

        # Get balances for all addresses
        results, total_balance = get_all_balances(
            node_url=args.node_url,
            contract_key=args.contract_key,
            address_map=address_map
        )

        # Print results in JSON format
        print("\nResults:")
        print(json.dumps(results, indent=2))

        # Print summary
        print("\nSummary:")
        print(f"Total addresses checked: {len(address_map)}")
        successful = sum(1 for v in results.values() if v["balance"] != "Address not found or balance unavailable")
        print(f"Successful queries: {successful}")
        print(f"Failed queries: {len(address_map) - successful}")
        print(f"Total balance: {format_large_number(total_balance)} tokens")

        # Detailed balance breakdown
        print("\nBalance Distribution:")
        sorted_balances = [
            (role, details) for role, details in results.items()
            if details["balance"] != "Address not found or balance unavailable"
        ]
        sorted_balances.sort(key=lambda x: float(x[1]["balance"]), reverse=True)
        
        for role, details in sorted_balances:
            print(f"{role}:")
            print(f"  Address: {details['address']}")
            print(f"  Balance: {format_large_number(float(details['balance']))} tokens")
            print()

    except Exception as e:
        print(f"Error in main: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
        sys.exit(0)
    except Exception as err:
        print(f"API ERROR :: {err}")
        sys.exit(1)
