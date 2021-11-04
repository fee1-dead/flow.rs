use std::error::Error;

use chrono::*;
use flow_sdk::prelude::*;
use flow_sdk::protobuf::Timestamp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut net = TonicHyperFlowClient::testnet().await?;
    let _ = net.ping().await?;
    let latest_block_header = net.latest_block_header(Seal::Sealed).await?;
    println!("{:?}", latest_block_header);
    let Timestamp { nanos, seconds } = latest_block_header.timestamp;
    println!("{}", Utc.timestamp(seconds, nanos as u32));

    println!("----------------");

    let block_header2 = net
        .block_header_by_height(latest_block_header.height)
        .await?;
    println!("{:?}", block_header2);
    let Timestamp { nanos, seconds } = block_header2.timestamp;
    println!("{}", Utc.timestamp(seconds, nanos as u32));

    assert_eq!(latest_block_header, block_header2);

    Ok(())
}
