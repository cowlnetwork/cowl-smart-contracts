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
    parser = argparse.ArgumentParser(description='Get Token Balances')
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
        '--addresses',
        nargs='+',
        required=True,
        help='List of public keys or account hashes to check balances for'
    )
    return parser.parse_args()

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

def get_all_balances(node_url: str, contract_key: str, addresses: list) -> Dict[str, str]:
    """Get balances for all addresses"""
    results = {}
    
    try:
        # Create node client (only once for all queries)
        client = NodeClient(NodeConnection(
            host=node_url,
            port_rpc=7777
        ))

        for address in addresses:
            balance = get_balance(client, address, contract_key)
            if balance is not None:
                results[address] = balance
            else:
                results[address] = "Address not found or balance unavailable"
                
        return results

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

def get_all_balances(node_url: str, contract_key: str, addresses: list) -> Dict[str, str]:
    """Get balances for all addresses"""
    results = {}
    total_balance = 0
    
    try:
        # Create node client (only once for all queries)
        client = NodeClient(NodeConnection(
            host=node_url,
            port_rpc=7777
        ))

        for address in addresses:
            balance = get_balance(client, address, contract_key)
            if balance is not None:
                results[address] = balance
                try:
                    total_balance += int(balance)
                except (ValueError, TypeError):
                    print(f"Warning: Could not add balance for {address} to total")
            else:
                results[address] = "Address not found or balance unavailable"
                
        return results, total_balance

    except Exception as e:
        print(f"Error connecting to node at {node_url}: {str(e)}")
        sys.exit(1)

def main():
    try:
        # Parse command line arguments
        args = parse_args()
        
        print(f"""
Token Balance Query
------------------
Node URL: {args.node_url}
Contract Key: {args.contract_key}
Number of addresses to check: {len(args.addresses)}
""")

        # Get balances for all addresses
        results, total_balance = get_all_balances(
            node_url=args.node_url,
            contract_key=args.contract_key,
            addresses=args.addresses
        )

        # Print results in JSON format
        print("\nResults:")
        print(json.dumps(results, indent=2))

        # Print summary
        print("\nSummary:")
        print(f"Total addresses checked: {len(args.addresses)}")
        successful = sum(1 for v in results.values() if v != "Address not found or balance unavailable")
        print(f"Successful queries: {successful}")
        print(f"Failed queries: {len(args.addresses) - successful}")
        print(f"Total balance: {format_large_number(total_balance)} tokens")

        # Detailed balance breakdown
        print("\nBalance Distribution:")
        sorted_balances = [
            (addr, bal) for addr, bal in results.items() 
            if bal != "Address not found or balance unavailable"
        ]
        sorted_balances.sort(key=lambda x: float(x[1]), reverse=True)
        
        for addr, balance in sorted_balances:
            print(f"{addr}: {format_large_number(float(balance))} tokens")

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
