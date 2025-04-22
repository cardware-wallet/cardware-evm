use cardware_evm::Wallet;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut wal = Wallet::new("zpub6qhLodRvBBKKmnMHWf3SUgucomzpcR8mRRw9V8sW9sEiLbVDwwN4N5y5tfojPWhKkuxyTtCsuv4W45q9MuxB95iFBVD2mXEyHrkmka1Woxq".to_string(),
                              "m/0/0".to_string(),
                              "https://cosmopolitan-proportionate-telescope.tron-mainnet.quiknode.pro/de5488c553ceb5e6d4cc52ca5648dd81467af58b/jsonrpc".to_string(),728126428); //Tron Chain ID: 728126428
    println!("Address: {:?}",wal.address());
    let res = wal.sync().await;
    println!("Res? {:?}",res);
    println!("Eth balance: {:?}",wal.balance());
    //let res = wal.send("TVD2kELrxNTFUEVVn4hgogSPbZeodrvMJZ".to_string(),"5000000",1);
    let res = wal.tron_send("TVD2kELrxNTFUEVVn4hgogSPbZeodrvMJZ".to_string(),"10000000",1).await;
    println!("RES: {:?}",res);

    //estimate fee example:
    let fee_est = wal.estimate_fee(1,21000);
    println!("fee est: {:?}",fee_est);
    let fee_est = wal.estimate_fee(1,160000);
    println!("fee est erc20: {:?}",fee_est);


    //Erc20 Balance
    //let erc_bal = wal.erc20_balance(["0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
    //                                  "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()].to_vec()).await;
    //println!("erc20 bal: {:?}",erc_bal);

    //Broadcast example
    //let res3 = wal.broadcast("ea80850cce4166008252089402a8665a18bba2d1b4766e2d71977a781b97592e857ea8ed40008081928080".to_string(),
    //                          "swUgntcRQS9BzpOfHNB9sZUbK5rwE0M9hvBAfSBohJlN6Xs2ijDS+UVz9U8/QRkYNN+yVp3y2ptBxFkaulVt8Rw=".to_string()).await; 
    //println!("res3: {:?}",res3); 

    //let erc_20_send = wal.erc20_transfer("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
    //                                   "0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),
    //                                   "1000000",2);

    //println!("erc 20 send: {:?}",erc_20_send);

    let res3 = wal.tron_broadcast("e88082013b82520894d307a7016456c639170cdc983d8328c6ae338b3b834c4b4080842b6653dc8080".to_string(),
                              "7fx8XYFKHVOvNXEwEN3HvFRVDN96ijn0H9nVp+dfn10V8X2kpK3g14bUh2wCeAo+yxRwr/pEmq+3oDUCNK2uKBs=".to_string()).await; 
    println!("res3: {:?}",res3); 
    //let con_data = wal.validate_contract("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()).await;
    //println!("contract data {}: ",con_data);
}
