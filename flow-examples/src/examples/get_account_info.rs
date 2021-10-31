use std::str::SplitWhitespace;

use anyhow::*;
use cadence_json::AddressOwned;
use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(
    account: &mut ExampleAccount,
    args: &mut SplitWhitespace<'_>,
) -> Result<()> {
    let addr = match args.next().map(str::parse::<AddressOwned>) {
        Some(Ok(addr)) => addr,
        Some(Err(_)) => bail!("Invalid argument 1: not an address"),
        None => bail!("Expected an argument (the address of the account), found none"),
    };

    let height = match args.next().map(str::parse::<u64>) {
        Some(Ok(height)) => Some(height),
        Some(Err(_)) => bail!("Invalid argument 2: not a number"),
        None => None,
    };

    let account = if let Some(height) = height {
        account
            .client()
            .account_at_block_height(&addr.data, height)
            .await?
    } else {
        account.client().account_at_latest_block(&addr.data).await?
    };

    println!("{:#?}", account);

    Ok(())
}
