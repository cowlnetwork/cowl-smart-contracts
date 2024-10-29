#!/usr/bin/env python3

import subprocess
import json
import os
from pathlib import Path
import argparse
from datetime import datetime
import shutil

class COWLAddressGenerator:
    def __init__(self, output_dir: str = "keys"):
        self.output_dir = output_dir
        self.keys_dir = Path(output_dir) / "keys"
        self.config_dir = Path(output_dir) / "config"
        self.backup_dir = Path(output_dir) / "backup"
        self.addresses = {}
        self.setup_directories()

    def setup_directories(self):
        """Create necessary directories."""
        for directory in [self.keys_dir, self.config_dir, self.backup_dir]:
            directory.mkdir(parents=True, exist_ok=True)

    def generate_key_pair(self, name: str) -> dict:
        """Generate a key pair using casper-client."""
        key_path = self.keys_dir / name
        try:
            # Generate keys
            result = subprocess.run(
                ["casper-client", "keygen", str(key_path)],
                capture_output=True,
                text=True,
                check=True
            )

            # Extract public key hash from output
            public_key_hash = None
            for line in result.stdout.split('\n'):
                if "Public Key Hash:" in line:
                    public_key_hash = line.split(":")[1].strip()

            if not public_key_hash:
                raise ValueError(f"Failed to extract public key hash for {name}")

            return {
                "name": name,
                "public_key_hash": public_key_hash,
                "secret_key_path": str(key_path / "secret_key.pem"),
                "public_key_path": str(key_path / "public_key.pem"),
                "account_hash": f"account-hash-{public_key_hash}"
            }
        except subprocess.CalledProcessError as e:
            print(f"Error generating keys for {name}: {e.stderr}")
            raise
        except Exception as e:
            print(f"Unexpected error generating keys for {name}: {str(e)}")
            raise

    def generate_all_addresses(self):
        """Generate all required addresses for COWL token deployment."""
        # Distribution addresses
        distribution_categories = [
            "treasury",
            "community_staking",
            "investor",
            "team",
            "network_rewards",
            "marketing",
            "community_rewards"
        ]

        # Multi-sig signers
        signer_count = 5
        signers = [f"signer_{i+1}" for i in range(signer_count)]

        print("\nGenerating Distribution Addresses:")
        for category in distribution_categories:
            print(f"Generating {category} address...")
            self.addresses[category] = self.generate_key_pair(category)
            print(f"✓ {category} address generated")

        print("\nGenerating Multi-sig Signer Addresses:")
fauthorized_signers = []
        for signer in signers:
            print(f"Generating {signer} address...")
            self.addresses[signer] = self.generate_key_pair(signer)
            authorized_signers.append(self.addresses[signer]["account_hash"])
            print(f"✓ {signer} address generated")

        # Create deployment configuration
        deployment_config = {
            "authorized_signers": authorized_signers,
            "distribution_addresses": {
                category: self.addresses[category]["account_hash"]
                for category in distribution_categories
            }
        }

        # Save deployment configuration
        config_path = self.config_dir / "deploy_config.json"
        with open(config_path, "w") as f:
            json.dump(deployment_config, f, indent=2)

        # Create detailed address report
        self.generate_address_report()
        self.backup_keys()

    def generate_address_report(self):
        """Generate a detailed report of all addresses and keys."""
        report = {
            "generated_at": datetime.now().isoformat(),
            "addresses": self.addresses,
            "keys_location": str(self.keys_dir.absolute()),
            "backup_location": str(self.backup_dir.absolute())
        }

        # Save detailed report
        report_path = self.config_dir / "address_report.json"
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)

        # Generate human-readable summary
        summary = ["COWL Token Address Generation Report", "=" * 40, ""]
        
        # Distribution addresses
        summary.append("Distribution Addresses:")
        summary.append("-" * 20)
        for category in ["treasury", "community_staking", "investor", "team", 
                        "network_rewards", "marketing", "community_rewards"]:
            if category in self.addresses:
                addr = self.addresses[category]
                summary.append(f"\n{category.upper()}:")
                summary.append(f"Account Hash: {addr['account_hash']}")
                summary.append(f"Public Key Hash: {addr['public_key_hash']}")
                summary.append(f"Key Location: {addr['secret_key_path']}")

        # Signer addresses
        summary.append("\nMulti-sig Signers:")
        summary.append("-" * 20)
        for i in range(5):
            signer = f"signer_{i+1}"
            if signer in self.addresses:
                addr = self.addresses[signer]
                summary.append(f"\n{signer.upper()}:")
                summary.append(f"Account Hash: {addr['account_hash']}")
                summary.append(f"Public Key Hash: {addr['public_key_hash']}")
                summary.append(f"Key Location: {addr['secret_key_path']}")

        # Save human-readable summary
        summary_path = self.config_dir / "address_summary.txt"
        with open(summary_path, "w") as f:
            f.write("\n".join(summary))

    def backup_keys(self):
        """Create a backup of all generated keys."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = self.backup_dir / f"keys_backup_{timestamp}"
        shutil.copytree(self.keys_dir, backup_path)
        print(f"\nKeys backed up to: {backup_path}")

    def verify_addresses(self):
        """Verify all generated addresses and keys."""
        print("\nVerifying generated addresses and keys...")
        
        for name, info in self.addresses.items():
            # Check if key files exist
            secret_key = Path(info["secret_key_path"])
            public_key = Path(info["public_key_path"])
            
            if not secret_key.exists():
                print(f"❌ Missing secret key for {name}")
                continue
            if not public_key.exists():
                print(f"❌ Missing public key for {name}")
                continue

            # Verify key pair
            try:
                result = subprocess.run(
                    ["casper-client", "account-address", 
                     "-p", str(public_key)],
                    capture_output=True,
                    text=True,
                    check=True
                )
                print(f"✓ Verified keys for {name}")
            except subprocess.CalledProcessError:
                print(f"❌ Invalid key pair for {name}")

def main():
    parser = argparse.ArgumentParser(
        description='Generate addresses for COWL token deployment')
    parser.add_argument('--output-dir', default='keys',
                      help='Directory to store generated keys and configs')
    parser.add_argument('--verify', action='store_true',
                      help='Verify generated addresses after creation')
    
    args = parser.parse_args()

    generator = COWLAddressGenerator(args.output_dir)
    
    try:
        print("Starting COWL address generation...")
        generator.generate_all_addresses()
        
        if args.verify:
            generator.verify_addresses()
            
        print("\nAddress generation completed successfully!")
        print(f"Configuration files located in: {generator.config_dir}")
        print(f"Key files located in: {generator.keys_dir}")
        print(f"Backup files located in: {generator.backup_dir}")
        
    except Exception as e:
        print(f"\nError during address generation: {str(e)}")
        return 1

    return 0

if __name__ == "__main__":
    exit(main())
