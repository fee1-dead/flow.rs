use std::error::Error;
use std::str::SplitWhitespace;

use crate::*;

crate::example!(run);

async fn run(
    account: &mut ExampleAccount,
    args: &mut SplitWhitespace<'_>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let id = match args.next().map(hex::decode) {
        Some(Ok(id)) => id,
        Some(Err(_)) => bail!("Invalid argument 1: not a hex encoded transaction id"),
        None => bail!("Expected an argument (the ID of the transaction), found none"),
    };

    let txn = account.client().transaction_result_by_id(&id).await?;

    println!("{:#?}", txn);

    Ok(())
}
