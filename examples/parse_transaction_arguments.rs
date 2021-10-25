use std::error::Error;

use flow_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::testnet()?;
    client.ping().await?;

    // traverse the blocks until we find collection guarantees
    let mut latest_block = client.latest_block(Seal::Sealed).await?.0;

    let collection_guarrantee = loop {
        if latest_block.collection_guarantees.is_empty() {
            // Go to the next block
            latest_block = client.block_by_id(&latest_block.parent_id).await?.0;
        } else {
            break latest_block.collection_guarantees.pop().unwrap();
        }
    };

    let collection = client
        .collection_by_id(&collection_guarrantee.collection_id)
        .await?
        .collection;

    for transaction_id in collection.transactions.iter() {
        let txn = client.transaction_by_id(transaction_id).await?.transaction;
        println!("{:#?}", txn);
        for argument in txn.parse_arguments() {
            println!("Found a cadence argument in the wild: {:#?}", argument?);
        }
    }

    Ok(())
}
