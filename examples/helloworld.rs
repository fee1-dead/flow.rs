use std::error::Error;

use flow_sdk::client::TonicHyperFlowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut net = TonicHyperFlowClient::mainnet()?;
    let _ = net.ping().await?;
    let latest_block_header = net.latest_block_header(true).await?;
    println!("{:?}", latest_block_header.into_inner().block);
    Ok(())
}