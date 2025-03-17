use cardware_evm::Wallet;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut wal = Wallet::new("zpub6qhLodRvBBKKmnMHWf3SUgucomzpcR8mRRw9V8sW9sEiLbVDwwN4N5y5tfojPWhKkuxyTtCsuv4W45q9MuxB95iFBVD2mXEyHrkmka1Woxq".to_string(),
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
    /*let res3 = wal.broadcast("e8808430fc4dae8252089402a8665a18bba2d1b4766e2d71977a781b97592e857ea8ed400080018080".to_string(),
                              "HEF58ia0Hr+7lzfMSH0QUw+EzN++IxPtST5KJ7sh4okWdaDf1bJS0Syu25/8E9Gd+4qOU5FvlFinrzTnJN/eOhw=".to_string()).await; 
    println!("res3: {:?}",res3); */

}
