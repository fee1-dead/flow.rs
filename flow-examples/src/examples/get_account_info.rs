use std::error::Error;
use std::str::SplitWhitespace;

use cadence_json::AddressOwned;
use flow_sdk::prelude::*;

use crate::*;

crate::example! {
    pub async fn run(account: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let addr = match args.next().map(str::parse::<AddressOwned>) {
            Some(Ok(addr)) => addr,
            Some(Err(_)) => bail!("Invalid argument: not an address"),
            None => bail!("Expected an argument (the address of the account), found none"),
        };

        let account = account.client().account_at_latest_block(&addr.data).await?;

        println!("{:#?}", account);

        Ok(())
    }
}