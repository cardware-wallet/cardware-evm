use cardware_evm::Wallet;

#[tokio::main]
async fn main() {
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
    /*
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
    
    // NEW: Transaction History Testing (Etherscan API with hardcoded key)
    println!("\n=== TESTING NEW TRANSACTION HISTORY FEATURES ===");
    
    // Test 1: Get nonce from Etherscan
    println!("Testing Etherscan nonce...");
    match wal.get_nonce_from_etherscan().await {
        Ok(nonce) => println!("Etherscan nonce: {}", nonce),
        Err(e) => println!("Etherscan nonce error: {}", e),
    }
    
    // Test 2: Sync with Etherscan for nonce comparison
    println!("Testing hybrid sync (Etherscan for nonce)...");
    let hybrid_sync = wal.sync_with_etherscan(true).await;
    println!("Hybrid sync result: {}", hybrid_sync);
    
    // Test 3: Get simple transaction history (last 5 transactions)
    println!("Testing simple transaction history...");
    let simple_tx = wal.get_simple_transaction_history(Some(5)).await;
    if simple_tx.len() > 100 {
        println!("Simple TX History (first 100 chars): {}...", &simple_tx[..100]);
    } else {
        println!("Simple TX History: {}", simple_tx);
    }
    
    // Test 4: Get raw transaction history (last 2 transactions)
    println!("Testing raw transaction history...");
    let raw_tx = wal.get_transaction_history(Some(2)).await;
    if raw_tx.len() > 150 {
        println!("Raw TX History (first 150 chars): {}...", &raw_tx[..150]);
    } else {
        println!("Raw TX History: {}", raw_tx);
    }
    
    println!("=== TRANSACTION HISTORY TESTS COMPLETE ===");
}