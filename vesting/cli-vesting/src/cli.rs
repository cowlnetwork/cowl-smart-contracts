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

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "list-addr")]
    ListFundedAdresses,
    #[command(name = "deploy")]
    DeployContracts {
        // #[arg(short, long, default_value = "mainnet")]
        // network: String,
    },
    #[command(name = "summary")]
    TokenDistributionSummary,
    #[command(name = "vesting")]
    VestingStatutes,
}
