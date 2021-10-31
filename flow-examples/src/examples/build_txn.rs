use std::fs;
use std::str::SplitWhitespace;

use anyhow::*;
use cadence_json::ValueOwned;
use flow_sdk::prelude::*;
use flow_sdk::transaction::TransactionHeader;
use tokio::sync::Mutex;

use crate::*;

crate::example!(run);

type TxnHeader = TransactionHeader<Vec<Box<[u8]>>>;

pub static BUILT_TXN: Mutex<Option<TxnHeader>> = Mutex::const_new(None);

async fn run(_: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<()> {
    let script_path = args
        .next()
        .with_context(|| "Expected a path to the script file")?;
    let arguments_path = args.next();

    let script = fs::read(script_path)
        .with_context(|| format!("While reading script from {}", script_path))?;
    let script = String::from_utf8(script)
        .with_context(|| format!("While reading script from {}", script_path))?;

    let arguments_raw: Option<Vec<u8>> = arguments_path
        .map(fs::read)
        .transpose()
        .with_context(|| format!("Opening {}", arguments_path.unwrap()))?;

    let arguments: Vec<ValueOwned> = arguments_raw
        .as_deref()
        .map(serde_json::from_slice)
        .transpose()
        .with_context(|| "Parsing arguments file as Cadence JSON")?
        .unwrap_or_default();

    let header = TransactionHeaderBuilder::new().script_owned(script).arguments(arguments).build();

    let mut built = BUILT_TXN.lock().await;
    if let Some(prev_txn) = built.take() {
        println!("Discarding previously built transaction: {:#?}", prev_txn);
    }
    println!("Built transaction: {:#?}", header);

    *built = Some(header);


    Ok(())
}
