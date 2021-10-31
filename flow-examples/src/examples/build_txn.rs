use std::error::Error;
use std::fs;
use std::str::SplitWhitespace;


use anyhow::*;
use cadence_json::ValueOwned;
use flow_sdk::access::*;
use flow_sdk::multi::{PartyBuilder, PartyTransaction};
use flow_sdk::prelude::*;
use flow_sdk::transaction::TransactionE;
use tokio::sync::Mutex;

use crate::*;

crate::example!(run);

type Txn = PartyTransaction<Box<[u8]>, [u8; 64]>;

pub static BUILT_TXN: Mutex<Option<Txn>> = Mutex::const_new(None);

async fn run(
    account: &mut ExampleAccount,
    args: &mut SplitWhitespace<'_>,
) -> Result<()> {
    let client = account.client();
    let script_path = args.next().with_context(|| "Expected a path to the script file")?;
    let script = fs::read(script_path).with_context(|| format!("While reading script from {}", script_path))?;

    let script = String::from_utf8(script).with_context(|| format!("While reading script from {}", script_path))?;

    let mut arguments_path = args.next();

    let arguments_raw: Option<Vec<u8>> = arguments_path
        .map(|p| fs::read(p))
        .transpose()
        .with_context(|| format!("Opening arguments file"))?;

    let arguments: Vec<ValueOwned> = arguments_raw.as_deref().map(serde_json::from_slice)
        .transpose()
        .with_context(|| format!("Parsing arguments file as Cadence JSON"))?
        .unwrap_or_default();

    let party = PartyBuilder::new()
        .script(script)
        .arguments(arguments)
        .authorizer_account(account)
        .payer_account(account)
        .proposer_account(account).await?
        .latest_block_as_reference(client).await?
        ;
   

    Ok(())
}
