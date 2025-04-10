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
    let res = wal.send("0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),"544000000000",1);
    println!("RES: {:?}",res);

    //estimate fee example:
    let fee_est = wal.estimate_fee(1,21000);
    println!("fee est: {:?}",fee_est);
    let fee_est = wal.estimate_fee(1,160000);
    println!("fee est erc20: {:?}",fee_est);


    //Erc20 Balance
    let erc_bal = wal.erc20_balance("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()).await;
    println!("erc20 bal: {:?}",erc_bal);

    //Broadcast example
    //let res3 = wal.broadcast("ea80850cce4166008252089402a8665a18bba2d1b4766e2d71977a781b97592e857ea8ed40008081928080".to_string(),
    //                          "swUgntcRQS9BzpOfHNB9sZUbK5rwE0M9hvBAfSBohJlN6Xs2ijDS+UVz9U8/QRkYNN+yVp3y2ptBxFkaulVt8Rw=".to_string()).await; 
    //println!("res3: {:?}",res3); 

    let erc_20_send = wal.erc20_transfer("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                                       "0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),
                                       "1000000",2);

    println!("erc 20 send: {:?}",erc_20_send);

    let res3 = wal.broadcast("f869078447596cda8302710094a0b86991c6218b36c1d19d4a2e9eb0ce3606eb4880b844a9059cbb00000000000000000000000002a8665a18bba2d1b4766e2d71977a781b97592e00000000000000000000000000000000000000000000000000000000000f4240018080".to_string(),
                              "XQdtleo1VPCj0YGpghoStvRZB2heI+4/c5X5PdN2T9tAQSsZSa1ssE0iEDW+KK1KQrEx2ko9Mv98QOae8c/UHRw=".to_string()).await; 
    println!("res3: {:?}",res3); 
    let con_data = wal.validate_contract("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()).await;
    println!("contract data {}: ",con_data);
}
