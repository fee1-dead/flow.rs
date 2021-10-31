use std::str::SplitWhitespace;

use anyhow::*;

use crate::ExampleAccount;

crate::example!(run);

async fn run(account: &mut ExampleAccount, _: &mut SplitWhitespace<'_>) -> Result<()> {
    let mut lock = crate::examples::build_txn::BUILT_TXN.lock().await;

    let txn = lock.take().with_context(|| {
        "No transaction was built. Please build a transaction using `run build_txn`."
    })?;

    drop(lock);

    account.send_transaction_header(&txn).await?;

    Ok(())
}
