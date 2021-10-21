use std::error::Error;

use flow_sdk::client::TonicHyperFlowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet()?;
    client.ping().await?;

    let latest_block_height = client.latest_block_header(true).await?.0.height;
    let start_height = latest_block_height - 100;

    println!(
        "Searching for accounts created within the last 100 blocks ({}-{})...",
        start_height, latest_block_height
    );

    let mut accounts = Vec::new();

    for events in client
        .events_for_height_range("flow.AccountCreated", start_height, latest_block_height)
        .await?
        .results
        .iter()
    {
        for event in events.events.iter() {
            let val: cadence_json::ValueOwned = serde_json::from_slice(&event.payload)?;

            if let cadence_json::ValueOwned::Event(c) = val {
                for field in c.fields.into_iter().filter(|f| f.name == "address") {
                    if let cadence_json::ValueOwned::Address(addr) = field.value {
                        accounts.push(client.account_at_latest_block(&addr.data).await?.account);
                    }
                }
            }
        }
    }

    println!("Found {} accounts.", accounts.len());

    if let Some(acc) = accounts.into_iter().max_by_key(|acc| acc.balance) {
        println!(
            "\nThe richest account is 0x{} with a balance of {}.",
            hex::encode(acc.address),
            acc.balance
        );
    }

    Ok(())
}
