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
    #[sol(rpc)]
    contract ZenithPair {
        event Sync(uint112 reserve0, uint112 reserve1);
    }

    #[sol(rpc)]
    contract ZenithRouter {
        function swapExactTokensForTokens(
            uint amountIn,
            uint amountOutMin,
            address[] calldata path,
            address to,
            uint deadline
        ) external returns (uint[] memory amounts);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = PrivateKeySigner::from_str(private_key)?;
    let wallet = EthereumWallet::from(signer);

    // 2. Connect WebSocket
    let rpc_url = "ws://127.0.0.1:8545";
    let ws = WsConnect::new(rpc_url);

    // 3. build Provider
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_ws(ws)
        .await?;

    println!("✅ Sniper Bot Connected!");

    // 4. address configuration
    let pair_address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    let router_address = address!("8A791620dd6260079BF849Dc5567aDC3F2FdC318");

    // Fake token address
    let token_a = address!("0000000000000000000000000000000000000001");
    let token_b = address!("0000000000000000000000000000000000000002");
    // Define recipient (Anvil account 0)
    let recipient = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

    // Example based contract
    let router = ZenithRouter::new(router_address, provider.clone());

    // 5. Monitor Sync events
    let filter = Filter::new()
        .address(pair_address)
        .event("Sync(uint112,uint112)");

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    println!("🎯 Sniper Scope Active...");

    while let Some(_log) = stream.next().await {
        println!("\n🚨 Signal Detected!");

        let amount_in = U256::from(1000);
        let amount_out_min = U256::from(0);
        let deadline = U256::from(1999999999);

        let path = vec![token_a, token_b];

        println!("🔫 Triggering Swap Transaction...");

        let tx_builder =
            router.swapExactTokensForTokens(amount_in, amount_out_min, path, recipient, deadline);

        let tx_result = tx_builder.send().await;

        match tx_result {
            Ok(pending_tx) => {
                println!("⏳ Transaction sent! Hash: {}", pending_tx.tx_hash());
                let receipt_result = pending_tx.get_receipt().await;
                match receipt_result {
                    Ok(receipt) => println!("✅ Mined in Block: {:?}", receipt.block_number),
                    Err(e) => println!("❌ Mining Failed: {:?}", e),
                }
            }
            Err(e) => {
                println!("❌ Transaction Error: {:?}", e);
            }
        }
    }

    Ok(())
}
