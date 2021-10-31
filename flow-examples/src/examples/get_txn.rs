use std::str::SplitWhitespace;

use crate::*;
use anyhow::*;

crate::example!(run);

async fn run(account: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<()> {
    let id = match args.next().map(hex::decode) {
        Some(Ok(id)) => id,
        Some(Err(_)) => bail!("Invalid argument 1: not a hex encoded transaction id"),
        None => bail!("Expected an argument (the ID of the transaction), found none"),
    };

    let txn = account.client().transaction_by_id(&id).await?;

    println!("{:#?}", txn);

    Ok(())
}
