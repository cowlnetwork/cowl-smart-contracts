use std::fmt::{self, Display};

use clap::{Parser, Subcommand};

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
    },
    #[command(name = "status")]
    VestingStatus,
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

            Commands::VestingInfo { vesting_type } => write!(f, "Vesting Info  {}", vesting_type),
            Commands::VestingStatus => write!(f, "Vesting Status"),
        }
    }
}
