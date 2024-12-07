use casper_rust_wasm_sdk::{types::verbosity::Verbosity, SDK};
use constants::RPC_ADDRESS;
use once_cell::sync::Lazy;
use std::io::Write;
use std::{
    env,
    fs::File,
    io::{self, Read},
    sync::{Arc, Mutex},
};

pub mod config;
pub mod constants;
pub mod keys;

pub static SDK_INSTANCE: Lazy<Mutex<Option<Arc<SDK>>>> = Lazy::new(|| Mutex::new(None));

// Function to retrieve or create the SDK instance
pub fn sdk() -> Arc<SDK> {
    let mut instance = SDK_INSTANCE.lock().unwrap();
    if instance.is_none() {
        let new_sdk = SDK::new(Some(RPC_ADDRESS.to_string()), Some(Verbosity::High));
        *instance = Some(Arc::new(new_sdk));
    }
    instance.clone().unwrap()
}

pub fn read_wasm_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let path_buf = env::current_dir()?;
    let mut relative_path_buf = path_buf.clone();
    relative_path_buf.push(file_path);
    let mut file = File::open(relative_path_buf)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn prompt_yes_no(question: &str) -> bool {
    loop {
        log::warn!("{} (y/n): ", question);
        io::stdout().flush().unwrap(); // Ensure the prompt is printed

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please answer with 'y' or 'n'"),
        }
    }
}
