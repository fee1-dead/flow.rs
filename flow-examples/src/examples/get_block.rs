use std::str::SplitWhitespace;

use anyhow::*;
use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(account: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<()> {
    let client = account.client();
    let block = match args.next() {
        Some(arg) if arg.len() == 64 => {
            let mut block_id = [0; 32];
            hex::decode_to_slice(arg, &mut block_id)
                .with_context(|| "Expected block height or hex encoded block ID")?;
            client.block_by_id(&block_id).await?
        }
        Some(height) => {
            let height = height
                .parse()
                .with_context(|| "Expected block height or hex encoded block ID")?;
            client.block_by_height(height).await?
        }
        None => client.latest_block(Seal::Sealed).await?,
    };

    println!("{:#?}", block);

    Ok(())
}
