use std::error::Error;
use std::fs;
use std::str::SplitWhitespace;

use cadence_json::ValueOwned;
use flow_sdk::access::*;
use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(
    account: &mut ExampleAccount,
    args: &mut SplitWhitespace<'_>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = account.client();
    let script_path = args.next().ok_or("Expected path to script file")?;
    let script = fs::read(script_path)
        .map_err(|e| format!("IO error while opening script file: {}", e))?;

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
        .map_err(|e| format!("IO error while opening arguments file: {}", e))?;

    let arguments: Vec<ValueOwned> = arguments_raw.as_deref().map(serde_json::from_slice)
        .transpose()
        .map_err(|e| format!("JSON error while parsing arguments file: {}", e))?
        .unwrap_or_default();

    let return_val = match block {
        Some(arg) if arg.len() == 64 => {
            let mut block_id = [0; 32];
            hex::decode_to_slice(arg, &mut block_id)
                .map_err(|_| "Expected block height or hex encoded block ID")?;

            client.send(ExecuteScriptAtBlockIdRequest { block_id, script, arguments }).await?
        }
        Some(height) => {
            let block_height = height
                .parse()
                .map_err(|_| "Expected block height or hex encoded block ID")?;
            client.send(ExecuteScriptAtBlockHeightRequest { block_height, script, arguments }).await?
        }
        None => client.send(ExecuteScriptAtLatestBlockRequest { script, arguments }).await?,
    };

    println!("Return Value: {:#?}", return_val.parse()?);

    Ok(())
}
