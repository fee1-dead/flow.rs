use std::error::Error;

use flow_sdk::{client::TonicHyperFlowClient, protobuf::access::Timestamp};

use chrono::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut net = TonicHyperFlowClient::mainnet()?;
    let _ = net.ping().await?;
    let latest_block_header = net.latest_block_header(true).await?;
    let block_header = latest_block_header.into_inner().0.into_inner();
    println!("{:?}", block_header);
    let Timestamp { nanos, seconds } = block_header.timestamp.into_inner();
    println!("{}", Utc.timestamp(seconds, nanos as u32));

    Ok(())
}