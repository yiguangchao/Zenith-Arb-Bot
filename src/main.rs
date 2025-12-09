// src/main.rs
use alloy::providers::{ProviderBuilder, Provider};
use eyre::Result;
use std::env;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let rpc_url = "http://127.0.0.1:8545".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let block_number = provider.get_block_number().await?;

    println!("✅ Connected to Anvil! Current Block: {}", block_number);

    Ok(())
}