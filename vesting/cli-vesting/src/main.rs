use clap::Parser;
use cli_vesting::{
    cli::{Cli, Commands},
    commands,
    utils::config,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    config::init().await;

    let cli = Cli::parse();
    match cli.command {
        Commands::ListFundedAdresses => commands::addresses::list_funded_addresses(),
        Commands::DeployContracts {} => {
            if let Err(e) = commands::deploy::deploy_all_contracts().await {
                eprintln!("Error deploying contracts: {}", e);
                std::process::exit(1);
            }
        }
        Commands::TokenDistributionSummary => commands::summary::token_distribution_summary(),
        Commands::VestingStatutes => commands::statutes::vesting_statutes(),
    }
}
