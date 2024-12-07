use chrono::Local;
use clap::Parser;
use cli_vesting::{
    cli::{Cli, Commands},
    commands,
    utils::config,
};
use env_logger::Builder;
use log::LevelFilter;
use std::{fs::OpenOptions, io::Write, sync::Mutex};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logger
    init_logger();

    // Initialize config
    config::init().await;

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
        Commands::VestingInfo { vesting_type } => {
            commands::info::print_vesting_info(
                vesting_type
                    .as_str()
                    .try_into()
                    .expect("Failed to convert vesting type"),
            )
            .await
        }
        Commands::VestingStatus => commands::status::vesting_status(),
    }
}

fn init_logger() {
    // Open or create the log file in append mode
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log")
        .expect("Failed to open log file");

    let log_file = Mutex::new(log_file);

    Builder::new()
        .filter(None, LevelFilter::Info)
        .format(move |buf, record| {
            // Lock the log file for writing
            let mut log_file = log_file.lock().unwrap();

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let log_message = format!("{} [{}] {}\n", timestamp, record.level(), record.args());

            // Write to the log file
            log_file
                .write_all(log_message.as_bytes())
                .expect("Failed to write log");

            // Also write to the default logger output (stdout by default)
            writeln!(buf, "{}", log_message.trim_end())
        })
        .init();
}
