use std::error::Error;

use flow_sdk::{client::TonicHyperFlowClient, protobuf::Timestamp};

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

    println!("----------------");

    let block_header2 = net.block_header_by_height(block_header.height).await?.into_inner().0.into_inner();
    println!("{:?}", block_header2);
    let Timestamp { nanos, seconds } = block_header2.timestamp.into_inner();
    println!("{}", Utc.timestamp(seconds, nanos as u32));

    assert_eq!(block_header, block_header2);

    Ok(())
}
