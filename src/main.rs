use cardware_evm::Wallet;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut wal = Wallet::new("zpub6qhLodRvBBKKmnMHWf3SUgucomzpcR8mRRw9V8sW9sEiLbVDwwN4N5y5tfojPWhKkuxyTtCsuv4W45q9MuxB95iFBVD2mXEyHrkmka1Woxq".to_string(),
                              "https://mainnet.infura.io/v3/498507efe2844f7cb9d8f25dd6e8f92b".to_string(),0);
    println!("Address: {:?}",wal.address());
    let res = wal.sync().await;
    println!("Res? {:?}",res);
    println!("Eth balance: {:?}",wal.balance());
    let res = wal.send("0x02A8665a18BBa2D1B4766e2D71977A781b97592e".to_string(),"544000000000",None);
    println!("RES: {:?}",res);
}
