use alloy::{
    primitives::address,
    providers::{Provider, ProviderBuilder, WsConnect}, // Introducing WebSocket connector
    rpc::types::eth::Filter,
    sol,
};
use eyre::Result;
use futures::stream::StreamExt;

// Define listening events
sol! {
    #[sol(rpc)]
    contract ZenithPair {
        // Sync event: triggered every time the reserve level changes (Mint/Burn/Swap)
        event Sync(uint112 reserve0, uint112 reserve1);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Change the connection protocol to WebSocket (ws://)
    let rpc_url = "ws://127.0.0.1:8545";

    // establish a long connection
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    println!("✅ WebSocket Connected!");

    // 2. Configure listening targets
    let pair_address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");

    // Create filter (Filter)
    let filter = Filter::new()
        // .address(pair_address)
        .event("Sync(uint112,uint112)");

    // 3. Subscription event stream (Subscribe)
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    println!("🎧 Listening for Sync events on Pair: {}", pair_address);
    println!("waiting for transactions...");

    // 4. Process event flow (Reactive Loop)
    while let Some(log) = stream.next().await {
        match log.log_decode::<ZenithPair::Sync>() {
            Ok(decoded) => {
                let event = decoded.inner.data;
                println!(
                    "🔔 Event Detected! New Reserves => Reserve0: {}, Reserve1: {}",
                    event.reserve0, event.reserve1
                );
                // --- 💡 Trigger arbitrage logic here ---
                // calculate_price(event.reserve0, event.reserve1);
                // if price_gap > threshold { execute_trade() }
            }
            Err(e) => println!("Error decoding log: {:?}", e),
        }
    }
    Ok(())
}
