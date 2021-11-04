use std::error::Error;

use flow_sdk::prelude::*;

async fn run(tx_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet().await?;
    client.ping().await?;

    let decoded_tx_id = hex::decode(tx_id)?;

    let txn = client.transaction_by_id(&decoded_tx_id).await?;
    println!("{:#?}", txn);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    run("97ff408cbcd6622ac1bf42a185b5cd36a2c6e0f86913649abcd35013581b771c").await
}
