use std::fs;
use std::str::SplitWhitespace;

use anyhow::*;
use cadence_json::ValueOwned;
use flow_sdk::access::*;
use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(
    account: &mut ExampleAccount,
    args: &mut SplitWhitespace<'_>,
) -> Result<()> {
    let client = account.client();
    let script_path = args.next().with_context(|| "Expected path to script file")?;
    let script = fs::read(script_path)
        .with_context(|| format!("IO error while opening {}", script_path))?;

    let mut arguments_path = args.next();
    let block = match args.next() {
        Some(arg) => Some(arg),
        None => {
            if matches!(arguments_path, Some(path) if Path::new(path).exists()) {
                // it is definitely a path, so we don't want to steal this argument.
                None
            } else {
                // definitely not a path, steal this argument and see if that is a block height or id.
                arguments_path.take()
            }
        }
    };

    let arguments_raw: Option<Vec<u8>> = arguments_path
        .map(|p| fs::read(p))
        .transpose()
        .with_context(|| format!("Opening arguments file"))?;

    let arguments: Vec<ValueOwned> = arguments_raw.as_deref().map(serde_json::from_slice)
        .transpose()
        .with_context(|| format!("Parsing arguments file as Cadence JSON"))?
        .unwrap_or_default();

    let return_val = match block {
        Some(arg) if arg.len() == 64 => {
            let mut block_id = [0; 32];
            hex::decode_to_slice(arg, &mut block_id)
                .with_context(|| "Expected block height or hex encoded block ID")?;

            client.send(ExecuteScriptAtBlockIdRequest { block_id, script, arguments }).await?
        }
        Some(height) => {
            let block_height = height
                .parse()
                .with_context(|| "Expected block height or hex encoded block ID")?;
            client.send(ExecuteScriptAtBlockHeightRequest { block_height, script, arguments }).await?
        }
        None => client.send(ExecuteScriptAtLatestBlockRequest { script, arguments }).await?,
    };

    println!("Return Value: {:#?}", return_val.parse()?);

    Ok(())
}
