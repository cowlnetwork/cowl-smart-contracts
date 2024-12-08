use crate::commands;
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
    #[command(name = "list-addr")]
    ListFundedAdresses,
    #[command(name = "deploy")]
    DeployContracts {
        #[clap(long)]
        token: bool, // Deploy only the token contract
        #[clap(long)]
        vesting: bool, // Deploy only the vesting contract
    },
    #[command(name = "info")]
    VestingInfo {
        #[clap(long)]
        vesting_type: String,
        #[clap(long)]
        call_entry_point: bool,
    },
    #[command(name = "status")]
    VestingStatus {
        #[clap(long)]
        vesting_type: String,
    },
    Balance {
        #[clap(long)]
        vesting_type: String,
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
        Commands::Balance { vesting_type } => {
            commands::balance::print_vesting_balance(
                vesting_type
                    .as_str()
                    .try_into()
                    .expect("Failed to convert vesting type"),
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
            Commands::Balance { vesting_type } => {
                write!(f, "COWL Balance for {vesting_type}",)
            }
        }
    }
}
