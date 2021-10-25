use std::error::Error;

use flow_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::testnet()?;
    client.ping().await?;

    let latest_block = client.latest_block(Seal::Sealed).await?;

    let block_by_id = client.block_by_id(&latest_block.id).await?;

    let block_by_height = client.block_by_height(latest_block.height).await?;

    assert_eq!(latest_block, block_by_id);
    assert_eq!(latest_block, block_by_height);

    println!("OK: {:#?}", latest_block);

    Ok(())
}
