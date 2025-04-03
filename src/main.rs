use cardware_evm::Wallet;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut wal = Wallet::new("zpub6qhLodRvBBKKmnMHWf3SUgucomzpcR8mRRw9V8sW9sEiLbVDwwN4N5y5tfojPWhKkuxyTtCsuv4W45q9MuxB95iFBVD2mXEyHrkmka1Woxq".to_string(),
                              "m/1/0".to_string(),
                              "https://mainnet.infura.io/v3/498507efe2844f7cb9d8f25dd6e8f92b".to_string(),1);
    println!("Address: {:?}",wal.address());
    let res = wal.sync().await;
    println!("Res? {:?}",res);
    println!("Eth balance: {:?}",wal.balance());
    let res = wal.send("0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),"544000000000",1);
    println!("RES: {:?}",res);
    //estimate fee example:
    let fee_est = wal.estimate_fee(1);
    println!("fee est: {:?}",fee_est);


    let erc_bal = wal.erc20_balance("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()).await;
    println!("erc20 bal: {:?}",erc_bal);
    //let res3 = wal.broadcast("e880842870a9918252089402a8665a18bba2d1b4766e2d71977a781b97592e857ea8ed400080018080".to_string(),
    //                          "vkeiYDUHvCej0kNIQe6FivwXH0ZxSDNxGvEBzbmAWMkiTVPnEIXUr4doFa+DWQsCm4EGDuL9onZQq8gXdqEGahs=".to_string()).await; 
    //println!("res3: {:?}",res3); 

    let erc_20_send = wal.erc20_transfer("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                                        "0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),
                                        "100000",1);

    println!("erc 20 send: {:?}",erc_20_send);

    //let res3 = wal.broadcast("f8680184227cdb3982ea6094dac17f958d2ee523a2206206994597c13d831ec780b844a9059cbb00000000000000000000000002a8665a18bba2d1b4766e2d71977a781b97592e00000000000000000000000000000000000000000000000000000000000186a0018080".to_string(),
    //                          "vkeiYDUHvCej0kNIQe6FivwXH0ZxSDNxGvEBzbmAWMkiTVPnEIXUr4doFa+DWQsCm4EGDuL9onZQq8gXdqEGahs=".to_string()).await; 
    //println!("res3: {:?}",res3); 
    let con_data = wal.validate_contract("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()).await;
    println!("");
    println!("contract data {}: ",con_data);
    println!("");
}
