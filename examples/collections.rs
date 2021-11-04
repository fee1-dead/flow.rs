use std::error::Error;

use flow_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet().await?;
    client.ping().await?;

    let mut latest_block = client.latest_block(Seal::Sealed).await?;

    // traverse latest blocks until we find a collection guarantee.
    let collection_guarrantee = loop {
        if latest_block.collection_guarantees.is_empty() {
            // Go to the next block
            latest_block = client.block_by_id(&latest_block.parent_id).await?;
        } else {
            break latest_block.collection_guarantees.pop().unwrap();
        }
    };

    // retrieve the collection by id.
    let collection = client
        .collection_by_id(&collection_guarrantee.collection_id)
        .await?;

    println!("OK: {:#?}", collection);

    Ok(())
}
