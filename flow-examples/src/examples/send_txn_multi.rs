use std::str::SplitWhitespace;

use anyhow::*;
use flow_sdk::access::SendTransactionRequest;

use crate::ExampleAccount;

crate::example!(run);

async fn run(account: &mut ExampleAccount, _: &mut SplitWhitespace<'_>) -> Result<()> {
    let mut lock = crate::examples::build_txn_multi::BUILT_TXN.lock().await;

    let transaction = lock.take().with_context(|| {
        "No transaction was built. Please build a transaction using `run build_txn_multisign`."
    })?;

    drop(lock);

    account.client().send(SendTransactionRequest {
        transaction
    }).await?;

    Ok(())
}
