use std::error::Error;
use std::str::SplitWhitespace;

use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(account: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = account.client();
    let block = match args.next() {
        Some(arg) => {
            if arg.len() == 64 {
                let mut block_id = [0; 32];
                hex::decode_to_slice(arg, &mut block_id).map_err(|_| "Expected hex encoded block ID")?;
                client.block_by_id(&block_id).await?
            } else {
                let height = arg.parse().map_err(|_| "Expected block height")?;
                client.block_by_height(height).await?
            }
        }
        None => client.latest_block(Seal::Sealed).await?,
    };

    println!("{:#?}", block);

    Ok(())
}