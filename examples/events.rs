use std::error::Error;

use flow_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet().await?;
    client.ping().await?;

    let latest_block_height = client.latest_block_header(Seal::Sealed).await?.height;
    let start_height = latest_block_height - 20;

    println!(
        "Searching for accounts created within the last 20 blocks ({}-{})...",
        start_height, latest_block_height
    );

    for events in client
        .events_for_height_range("flow.AccountCreated", start_height, latest_block_height)
        .await?
        .results
        .iter()
    {
        if events.events.is_empty() {
            continue;
        }
        println!(
            "\nBlock #{} ({}):",
            events.block_height,
            hex::encode(&events.block_id)
        );
        for event in events.events.iter() {
            let val = event.parse_payload()?;

            println!("  - {:#?}", val);
        }
    }

    Ok(())
}
