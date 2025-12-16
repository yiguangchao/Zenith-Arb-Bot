use alloy::{
    network::EthereumWallet,
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::eth::Filter,
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::Result;
use futures::stream::StreamExt;
use std::str::FromStr;

sol! {
    // monitoring target
    #[sol(rpc)]
    contract ZenithPair {
        event Sync(uint112 reserve0, uint112 reserve1);
    }

    // excute target
    #[sol(rpc)]
    contract FlashArbitrage {
        function startArbitrage(address pairAddress, uint amount0Out, uint amount1Out) external;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // --- 2. Configure wallet (Anvil Account 0) ---
    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = PrivateKeySigner::from_str(private_key)?;
    let wallet = EthereumWallet::from(signer);

    // --- 3. connect WebSocket ---
    let rpc_url = "ws://127.0.0.1:8545";
    let ws = WsConnect::new(rpc_url);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_ws(ws)
        .await?;

    println!("✅ Arbitrage Bot Connected!");

    // --- 4. Configure address  ---
    let pair_address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    let flash_arb_address = address!("Dc64a140Aa3E981100a9becA4E685f962f0cF6C9");

    // Example based contract
    let flash_bot = FlashArbitrage::new(flash_arb_address, provider.clone());

    let filter = Filter::new()
        //    .address(pair_address)
        .event("Sync(uint112,uint112)");

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    println!("🎯 Waiting for signals...");

    while let Some(_log) = stream.next().await {
        println!("\n🚨 Signal Detected! Executing Flash Loan...");

        let amount0 = U256::from(10_000_000_000_000_000_000_u128);
        let amount1 = U256::from(0);

        let tx_builder = flash_bot.startArbitrage(pair_address, amount0, amount1);

        let tx = tx_builder.send().await;

        match tx {
            Ok(pending) => {
                println!("🚀 Transaction Broadcasted! Hash: {}", pending.tx_hash());
            }
            Err(e) => {
                println!("⚠️ Transaction Failed (As Expected): {}", e);
            }
        }
    }

    Ok(())
}
