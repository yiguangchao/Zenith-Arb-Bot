use alloy::{
    network::EthereumWallet,
    primitives::{address, U256}, // 记得引入 U256
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::eth::Filter,
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::Result;
use futures::stream::StreamExt;
use std::str::FromStr;

// --- 1. 定义 Solidity 接口 ---
sol! {
    // 监听目标
    #[sol(rpc)]
    contract ZenithPair {
        event Sync(uint112 reserve0, uint112 reserve1);
    }

    // 执行目标 (你的雇佣兵合约)
    #[sol(rpc)]
    contract FlashArbitrage {
        function startArbitrage(address pairAddress, uint amount0Out, uint amount1Out) external;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // --- 2. 配置钱包 (Anvil Account 0) ---
    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = PrivateKeySigner::from_str(private_key)?;
    let wallet = EthereumWallet::from(signer);

    // --- 3. 连接 WebSocket ---
    let rpc_url = "ws://127.0.0.1:8545";
    let ws = WsConnect::new(rpc_url);
    
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_ws(ws)
        .await?;

    println!("✅ Arbitrage Bot Connected!");

    // --- 4. 配置地址 (关键步骤) ---
    // ⚠️ 请务必替换为你当前环境的真实地址！
    let pair_address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"); 
    let flash_arb_address = address!("Dc64a140Aa3E981100a9becA4E685f962f0cF6C9");

    // 实例化合约
    let flash_bot = FlashArbitrage::new(flash_arb_address, provider.clone());

    // --- 5. 监听 Sync 事件 ---
    let filter = Filter::new()
        .address(pair_address)
        .event("Sync(uint112,uint112)");

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    println!("🎯 Waiting for signals...");

    // --- 6. 事件循环 ---
    // --- 6. 事件循环 ---
    while let Some(_log) = stream.next().await {
        println!("\n🚨 Signal Detected! Executing Flash Loan...");
        
        let amount0 = U256::from(10_000_000_000_000_000_000_u128); 
        let amount1 = U256::from(0);

        // --- 修复方案 ---
        
        // 1. 先创建 Builder 并赋值给变量 `tx_builder`
        // 这样它的生命周期就延长了，不会立即被销毁
        let tx_builder = flash_bot.startArbitrage(pair_address, amount0, amount1);

        // 2. 再调用 send()
        // 此时 tx_builder 还活着，send() 可以安全地引用它
        let tx = tx_builder.send().await;
        
        // --- 修复结束 ---

        match tx {
            Ok(pending) => {
                println!("🚀 Transaction Broadcasted! Hash: {}", pending.tx_hash());
            }
            Err(e) => {
                // 如果出现 Execution Reverted (因为没钱还贷)，这是预期内的
                println!("⚠️ Transaction Failed (As Expected): {}", e);
            }
        }
    }

    Ok(())
}