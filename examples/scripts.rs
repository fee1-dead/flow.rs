use std::error::Error;

use cadence_json::{AddressRef, ValueRef};
use flow_sdk::client::TonicHyperFlowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet()?;
    client.ping().await?;

    let latest_block_height = client.latest_block_header(true).await?.0.height;
    let start_height = latest_block_height - 100;

    println!("Searching for accounts created within the last 100 blocks ({}-{})...", start_height, latest_block_height);

    let mut richest_account = None;
    let mut richest_balance = None;

    for events in client.events_for_height_range("flow.AccountCreated", start_height, latest_block_height).await?.results.iter() {
        for event in events.events.iter() {
            let val: cadence_json::ValueOwned = serde_json::from_slice(&event.payload)?;

            if let cadence_json::ValueOwned::Event(c) = val {
                for field in c.fields.into_iter().filter(|f| f.name == "address") {
                    if let cadence_json::ValueOwned::Address(addr) = field.value {
                        let account = client.account_at_latest_block(&addr.data).await?.account;
                        if Some(account.balance) > richest_balance {
                            richest_balance = Some(account.balance);
                            richest_account = Some(account);
                        }
                    }
                }
            }
        }
    }

    if let Some(acc) = richest_account {
        println!("\nThe richest account is 0x{}.", hex::encode(&acc.address));
        let value = ValueRef::Address(AddressRef { data: &acc.address });
        let encoded = serde_json::to_vec(&value).unwrap();
        
        let res = client.execute_script_at_latest_block(b"
            pub fun main(address: Address): PublicAccount {
                return getAccount(address)
            }
        ", &[ &encoded ]).await?;
        println!("{:?}", res.parse()?);
    }

    Ok(())
}