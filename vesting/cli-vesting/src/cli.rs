use crate::{
    commands,
    utils::{config::get_key_pair_from_vesting, constants::COWL_CEP_18_TOKEN_SYMBOL},
};
use casper_rust_wasm_sdk::{
    helpers::motes_to_cspr,
    types::{key::Key, public_key::PublicKey},
};
use clap::{Parser, Subcommand};
use std::fmt::{self, Display};

/// CLI Tool for managing contracts and token distributions
#[derive(Parser)]
#[command(name = "cli-vesting")]
#[command(version = "1.0")]
#[command(about = "A CLI for Cowl token vesting admin", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List funded addresses stored in the configuration.
    #[command(name = "list-addr", about = "List all funded addresses")]
    ListFundedAdresses,

    /// Deploy smart contracts.
    #[command(
        name = "deploy",
        about = "Deploy contracts for the token or vesting or both by default"
    )]
    DeployContracts {
        /// Deploy only the token contract.
        #[arg(long, help = "Deploy only the token contract")]
        token: bool,

        /// Deploy only the vesting contract.
        #[arg(long, help = "Deploy only the vesting contract")]
        vesting: bool,
    },

    /// Retrieve vesting information.
    #[command(
        name = "info",
        about = "Get base vesting information about a vesting type"
    )]
    VestingInfo {
        /// Specify the vesting type (e.g., linear, cliff).
        #[arg(long, help = "The vesting type to retrieve information for")]
        vesting_type: String,

        /// Call the entry point in the contract for more detailed information.
        #[arg(long, help = "Call the contract's entry point before")]
        call_entry_point: bool,
    },

    /// Check the status of a vesting type.
    #[command(name = "status", about = "Check the current status of a vesting type")]
    VestingStatus {
        /// Specify the vesting type (e.g., linear, cliff).
        #[arg(
            long,
            help = "The vesting type to check the status for. Contract's entrypoint will be called before to update the value to retrieve."
        )]
        vesting_type: String,
    },

    /// Retrieve the balance of a vesting or public key.
    #[command(
        name = "balance",
        about = "Retrieve the balance for a specific vesting type or key (Public key or Account hash)"
    )]
    Balance {
        /// Specify the vesting type (optional).
        #[arg(long, help = "The vesting type to retrieve the balance for")]
        vesting_type: Option<String>,

        /// Specify the public or account key (optional).
        #[arg(
            long,
            help = "The public key or account hash to retrieve the balance for"
        )]
        key: Option<String>,
    },
    /// Transfer tokens between accounts or vesting types.
    #[command(
        name = "transfer",
        about = format!("Transfer {} tokens between accounts or vesting types", *COWL_CEP_18_TOKEN_SYMBOL)
    )]
    Transfer {
        /// Specify the source (vesting type or public key).
        #[arg(
            long,
            help = "The source (vesting type or public key/account hash) to transfer from"
        )]
        from: String,

        /// Specify the destination (vesting type or public key).
        #[arg(
            long,
            help = "The destination (vesting type or public key/account hash) to transfer to"
        )]
        to: String,
        #[arg(long, help = "The amount to transfer")]
        amount: String,
    },
    Allowance {
        #[arg(
            long,
            help = "The owner (vesting type or public key/account hash) of the allowance"
        )]
        owner: String,

        /// Specify the destination (vesting type or public key).
        #[arg(
            long,
            help = "The spender (vesting type or public key/account hash) of the allowance"
        )]
        spender: String,
    },
    TransferFrom {
        /// Specify the source (vesting type or public key).
        #[arg(
            long,
            help = "The allowed operator (vesting type or public key/account hash) to transfer from"
        )]
        operator: String,

        /// Specify the source (vesting type or public key).
        #[arg(
            long,
            help = "The source (vesting type or public key/account hash) to transfer from"
        )]
        from: String,

        /// Specify the destination (vesting type or public key).
        #[arg(
            long,
            help = "The destination (vesting type or public key/account hash) to transfer to"
        )]
        to: String,
        #[arg(long, help = "The amount to transfer")]
        amount: String,
    },
    IncreaseAllowance {
        #[arg(
            long,
            help = "The owner (vesting type or public key/account hash) of the tokens"
        )]
        owner: String,
        #[arg(
            long,
            help = "The spender (vesting type or public key/account hash) to increase allowance of"
        )]
        spender: String,
        #[arg(long, help = "The amount to increase")]
        amount: String,
    },
    DecreaseAllowance {
        #[arg(
            long,
            help = "The owner (vesting type or public key/account hash) of the tokens"
        )]
        owner: String,
        #[arg(
            long,
            help = "The spender (vesting type or public key/account hash) to decrease allowance of"
        )]
        spender: String,
        #[arg(long, help = "The amount to decrease")]
        amount: String,
    },
}

pub async fn run() {
    let cli = Cli::parse();

    log::info!("Command executed: {}", cli.command);

    match cli.command {
        Commands::ListFundedAdresses => commands::addresses::print_funded_addresses().await,
        Commands::DeployContracts { token, vesting } => {
            let result = if token {
                commands::deploy::deploy_cep18_token().await
            } else if vesting {
                commands::deploy::deploy_vesting_contract().await
            } else {
                commands::deploy::deploy_all_contracts().await
            };

            if let Err(e) = result {
                log::error!("Error deploying contracts: {}", e);
                std::process::exit(1);
            }
        }
        Commands::VestingInfo {
            vesting_type,
            call_entry_point,
        } => {
            commands::info::print_vesting_info(
                vesting_type
                    .as_str()
                    .try_into()
                    .expect("Failed to convert vesting type"),
                call_entry_point,
            )
            .await
        }
        Commands::VestingStatus { vesting_type } => {
            let call_entry_point = true; // Always call entry point before getting status
            commands::status::print_vesting_status(
                vesting_type
                    .as_str()
                    .try_into()
                    .expect("Failed to convert vesting type"),
                call_entry_point,
            )
            .await
        }
        Commands::Balance { vesting_type, key } => {
            commands::balance::print_balance(
                vesting_type.map(|f| {
                    f.as_str()
                        .try_into()
                        .expect("Failed to convert vesting type")
                }),
                key.map(|formatted_str| parse_key_from_formatted_str(&formatted_str)),
            )
            .await
        }
        Commands::Transfer { from, to, amount } => {
            commands::transfer::print_transfer(
                PublicKey::new(&from).expect("Failed to convert public key to key"),
                parse_key_from_formatted_str(&to),
                amount,
            )
            .await
        }
        Commands::Allowance { owner, spender } => {
            // Retrieve the key pair for the owner
            let owner_key = if let Some(key_pair) = get_key_pair_from_vesting(&owner).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&owner)
            };

            // Retrieve the key pair for the spender
            let spender_key = if let Some(key_pair) = get_key_pair_from_vesting(&spender).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&spender)
            };

            commands::allowance::print_get_allowance(&owner_key, &spender_key).await
        }
        Commands::TransferFrom {
            operator,
            from,
            to,
            amount,
        } => {
            commands::transfer_from::print_transfer_from(
                PublicKey::new(&operator).expect("Failed to convert public key to key"),
                parse_key_from_formatted_str(&from),
                parse_key_from_formatted_str(&to),
                amount,
            )
            .await
        }
        Commands::IncreaseAllowance {
            owner,
            spender,
            amount,
        } => {
            // Retrieve the key pair for the spender
            let spender_key = if let Some(key_pair) = get_key_pair_from_vesting(&spender).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&spender)
            };

            commands::allowance::print_increase_allowance(
                &PublicKey::new(&owner).expect("Failed to convert public key to key"),
                &spender_key,
                amount,
            )
            .await
        }
        Commands::DecreaseAllowance {
            owner,
            spender,
            amount,
        } => {
            // Retrieve the key pair for the spender
            let spender_key = if let Some(key_pair) = get_key_pair_from_vesting(&spender).await {
                Key::from_account(key_pair.public_key.to_account_hash())
            } else {
                parse_key_from_formatted_str(&spender)
            };
            commands::allowance::print_decrease_allowance(
                &PublicKey::new(&owner).expect("Failed to convert public key to key"),
                &spender_key,
                amount,
            )
            .await
        }
    }
}

impl Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Commands::ListFundedAdresses => write!(f, "List Funded Adresses"),

            Commands::DeployContracts { token, vesting } => {
                if *token || *vesting {
                    write!(
                        f,
                        "Deploy Contracts {{ token: {}, vesting: {} }}",
                        token, vesting
                    )
                } else {
                    write!(f, "Deploy All Contracts {{ token: true, vesting: true }}")
                }
            }

            Commands::VestingInfo {
                vesting_type,
                call_entry_point: _,
            } => write!(f, "Vesting Info for {vesting_type}",),
            Commands::VestingStatus { vesting_type } => {
                write!(f, "Vesting Status for {vesting_type}",)
            }
            Commands::Balance { vesting_type, key } => {
                if let Some(vesting_type) = vesting_type {
                    write!(
                        f,
                        "{} Balance for {}",
                        *COWL_CEP_18_TOKEN_SYMBOL, vesting_type
                    )
                } else if let Some(key) = key {
                    write!(f, "{} Balance for {}", *COWL_CEP_18_TOKEN_SYMBOL, key)
                } else {
                    write!(
                        f,
                        "{} Balance: No vesting_type or key provided",
                        *COWL_CEP_18_TOKEN_SYMBOL
                    )
                }
            }
            Commands::Transfer { from, to, amount } => {
                write!(
                    f,
                    "Transfer {} {} \nfrom {} \nto: {}",
                    motes_to_cspr(amount).unwrap(),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    from.clone(),
                    to.clone()
                )
            }
            Commands::Allowance { owner, spender } => {
                write!(
                    f,
                    "{} Allowance \nfrom {owner} \nto {spender}",
                    *COWL_CEP_18_TOKEN_SYMBOL
                )
            }
            Commands::TransferFrom {
                operator,
                from,
                to,
                amount,
            } => {
                write!(
                    f,
                    "Transfer {} {} \nby{} \nfrom {} \nto: {}",
                    motes_to_cspr(amount).unwrap(),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    operator.clone(),
                    from.clone(),
                    to.clone()
                )
            }
            Commands::IncreaseAllowance {
                owner: _,
                spender,
                amount,
            } => {
                write!(
                    f,
                    "IncreaseAllowance {} {} \nfrom {}",
                    motes_to_cspr(amount).unwrap(),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    spender.clone(),
                )
            }
            Commands::DecreaseAllowance {
                owner: _,
                spender,
                amount,
            } => {
                write!(
                    f,
                    "DecreaseAllowance {} {} \nfrom {}",
                    motes_to_cspr(amount).unwrap(),
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    spender.clone(),
                )
            }
        }
    }
}

fn parse_key_from_formatted_str(formatted_str: &str) -> Key {
    if let Ok(public_key) = PublicKey::new(formatted_str) {
        Key::from_formatted_str(&public_key.to_account_hash().to_formatted_string())
            .expect("Failed to convert public key to key")
    } else {
        Key::from_formatted_str(formatted_str).expect("Failed to convert key")
    }
}
