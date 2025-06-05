use cardware_evm::Wallet;

#[tokio::main]
async fn main() {
    println!("=== Testing Etherscan API Integration ===");
    
    // Test Etherscan API with the provided address
    let _test_address = "0x0122783b377fbd437880334872b85f3ba9a5f99d";
    let etherscan_api_key = "KAQABZ3CB12ETJC8QG6WT3DRI2IH95I8I7";
    
    // Create a dummy wallet for testing (we'll override the address)
    let mut wal = Wallet::new(
        "zpub6qhLodRvBBKKmnMHWf3SUgucomzpcR8mRRw9V8sW9sEiLbVDwwN4N5y5tfojPWhKkuxyTtCsuv4W45q9MuxB95iFBVD2mXEyHrkmka1Woxq".to_string(),
        "m/0/0".to_string(),
        "https://mainnet.infura.io/v3/498507efe2844f7cb9d8f25dd6e8f92b".to_string(),
        1
    );
    
    // Set the Etherscan API key
    wal.set_etherscan_api_key(etherscan_api_key.to_string());
    println!("✓ Etherscan API key set");
    
    // Get the derived address
    let derived_address = wal.address();
    println!("Derived address from xpub: {}", derived_address);
    
    // Test 1: Get nonce for the derived address using Etherscan
    println!("\n=== Test 1: Getting nonce for derived address ===");
    match wal.get_nonce_from_etherscan().await {
        Ok(nonce) => println!("✓ Nonce from Etherscan: {}", nonce),
        Err(e) => println!("✗ Failed to get nonce: {}", e),
    }
    
    // Test 2: Sync with Etherscan for nonce
    println!("\n=== Test 2: Sync wallet using Etherscan for nonce ===");
    let sync_result = wal.sync_with_etherscan(true).await;
    println!("Sync result: {}", sync_result);
    
    if sync_result == "Sync successful." {
        println!("✓ Wallet synced successfully!");
        println!("  - ETH Balance: {}", wal.balance());
        println!("  - Nonce: {}", wal.get_nonce());
        println!("  - Chain ID: {}", wal.get_chain_id());
    }
    
    // Test 3: Get transaction history
    println!("\n=== Test 3: Getting transaction history ===");
    let tx_history = wal.get_simple_transaction_history(Some(5)).await;
    
    if tx_history.starts_with("Error:") {
        println!("✗ Transaction history error: {}", tx_history);
    } else if tx_history == "[]" {
        println!("✓ No transactions found for this address");
    } else {
        println!("✓ Transaction history retrieved successfully!");
        
        // Parse and display the transactions nicely
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tx_history) {
            if let Some(txs) = parsed.as_array() {
                println!("Found {} recent transactions:", txs.len());
                
                for (i, tx) in txs.iter().enumerate() {
                    let hash = tx.get("hash").and_then(|h| h.as_str()).unwrap_or("N/A");
                    let direction = tx.get("direction").and_then(|d| d.as_str()).unwrap_or("N/A");
                    let value_eth = tx.get("value_eth").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let from = tx.get("from").and_then(|f| f.as_str()).unwrap_or("N/A");
                    let to = tx.get("to").and_then(|t| t.as_str()).unwrap_or("N/A");
                    let block = tx.get("block_number").and_then(|b| b.as_str()).unwrap_or("N/A");
                    
                    println!("  {}. {} {} ETH - {} (Block: {})", 
                        i + 1, direction, value_eth, &hash[..10], block);
                    println!("     From: {}...", &from[..10]);
                    println!("     To:   {}...", &to[..10]);
                }
            }
        }
    }
    
    // Test 4: Get full transaction history (raw data)
    println!("\n=== Test 4: Getting full transaction history (first 3) ===");
    let full_history = wal.get_transaction_history(Some(3)).await;
    
    if full_history.starts_with("Error:") {
        println!("✗ Full transaction history error: {}", full_history);
    } else if full_history == "[]" {
        println!("✓ No transactions found");
    } else {
        println!("✓ Full transaction history retrieved successfully!");
        // Just show first 200 characters to avoid overwhelming output
        let preview = if full_history.len() > 200 {
            format!("{}...", &full_history[..200])
        } else {
            full_history
        };
        println!("Raw data preview: {}", preview);
    }
    
    // Test 5: Compare with regular sync (using Infura for nonce)
    println!("\n=== Test 5: Compare with regular Infura sync ===");
    let infura_sync_result = wal.sync().await;
    println!("Infura sync result: {}", infura_sync_result);
    
    if infura_sync_result == "Sync successful." {
        println!("✓ Infura sync successful!");
        println!("  - Nonce from Infura: {}", wal.get_nonce());
    }
    
    println!("\n=== Etherscan API Test Complete ===");
    
    /*
    // Original test code (commented out for now)
    println!("Hello, world!");
    let mut wal = Wallet::new("zpub6qhLodRvBBKKmnMHWf3SUgucomzpcR8mRRw9V8sW9sEiLbVDwwN4N5y5tfojPWhKkuxyTtCsuv4W45q9MuxB95iFBVD2mXEyHrkmka1Woxq".to_string(),
                              "m/0/0".to_string(),
                              "https://mainnet.infura.io/v3/498507efe2844f7cb9d8f25dd6e8f92b".to_string(),1);
    println!("Address: {:?}",wal.address());
    let res = wal.sync().await;
    println!("Res? {:?}",res);
    
    println!("Eth balance: {:?}",wal.balance());
    let res = wal.send_eip1559("0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),"544000000000",1);
    println!("RES: {:?}",res);

    let res5 = wal.prepare_sign_typed_data_v4("{\"domain\":{\"name\":\"Permit2\",\"chainId\":1,\"verifyingContract\":\"0x31c2f6fcff4f8759b3bd5bf0e1084a055615c768\"},\"message\":{\"details\":{\"token\":\"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\",\"amount\":\"1461501637330902918203684832716283019655932542975\",\"expiration\":\"1750587227\",\"nonce\":\"0\"},\"spender\":\"0x65b382653f7c31bc0af67f188122035461ec9c76\",\"sigDeadline\":\"1747997027\"},\"primaryType\":\"PermitSingle\",\"types\":{\"EIP712Domain\":[{\"name\":\"name\",\"type\":\"string\"},{\"name\":\"chainId\",\"type\":\"uint256\"},{\"name\":\"verifyingContract\",\"type\":\"address\"}],\"PermitSingle\":[{\"name\":\"details\",\"type\":\"PermitDetails\"},{\"name\":\"spender\",\"type\":\"address\"},{\"name\":\"sigDeadline\",\"type\":\"uint256\"}],\"PermitDetails\":[{\"name\":\"token\",\"type\":\"address\"},{\"name\":\"amount\",\"type\":\"uint160\"},{\"name\":\"expiration\",\"type\":\"uint48\"},{\"name\":\"nonce\",\"type\":\"uint48\"}]}}".to_string());
    println!("EIP712 res: {:?}",res5);
    //estimate fee example:
    //Erc20 Balance
    
    let erc_bal = wal.erc20_balance(["0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                                      "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()].to_vec()).await;
    println!("erc20 bal: {:?}",erc_bal);

    //Broadcast example
    //let res3 = wal.broadcast("ea80850cce4166008252089402a8665a18bba2d1b4766e2d71977a781b97592e857ea8ed40008081928080".to_string(),
    //                          "swUgntcRQS9BzpOfHNB9sZUbK5rwE0M9hvBAfSBohJlN6Xs2ijDS+UVz9U8/QRkYNN+yVp3y2ptBxFkaulVt8Rw=".to_string()).await; 
    //println!("res3: {:?}",res3); 

    let erc_20_send = wal.erc20_transfer("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                                       "0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),
                                       "1000000",2);

    println!("erc 20 send: {:?}",erc_20_send);

    //let res3 = wal.broadcast("f869078447596cda8302710094a0b86991c6218b36c1d19d4a2e9eb0ce3606eb4880b844a9059cbb00000000000000000000000002a8665a18bba2d1b4766e2d71977a781b97592e00000000000000000000000000000000000000000000000000000000000f4240018080".to_string(),
    //                          "XQdtleo1VPCj0YGpghoStvRZB2heI+4/c5X5PdN2T9tAQSsZSa1ssE0iEDW+KK1KQrEx2ko9Mv98QOae8c/UHRw=".to_string()).await; 
    //println!("res3: {:?}",res3); 
    //let con_data = wal.validate_contract("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()).await;
    //println!("contract data {}: ",con_data);
    println!("nonce: {:?}",wal.get_nonce());
    println!("chain_id: {:?}",wal.get_chain_id());
    println!("hex to b64: {:?}",wal.hex_to_b64("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()));
    let res3 = wal.construct_signed_tx("0x02f8700108847735940085017e73c1dc82fe7b94a0b86991c6218b36c1d19d4a2e9eb0ce3606eb4880b844095ea7b3000000000000000000000000000000000022d473030f116ddee9f6b43ac78ba3ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc0808080".to_string(),
                                      "148v8u32WP1MW3JOk/Iz1OOKMZ8jLO4An1jcfGzx7AksGQtP+dm8lw6Zri94txhV+gQEeMrnvleImffPG1wQkRs=".to_string());
    println!("reso {:?}",res3);
    */
}