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
    #[command(name = "summary")]
    TokenDistributionSummary,
    #[command(name = "vesting")]
    VestingStatutes,
}

impl Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Commands::ListFundedAdresses => write!(f, "ListFundedAdresses"),

            Commands::DeployContracts { token, vesting } => {
                if token || vesting {
                    write!(
                        f,
                        "Deploy Contracts {{ token: {}, vesting: {} }}",
                        token, vesting
                    )
                } else {
                    write!(f, "Deploy All Contracts {{ token: true, vesting: true }}")
                }
            }

            Commands::TokenDistributionSummary => write!(f, "TokenDistributionSummary"),
            Commands::VestingStatutes => write!(f, "VestingStatutes"),
        }
    }
}
