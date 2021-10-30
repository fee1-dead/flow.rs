use std::error::Error;
use std::str::SplitWhitespace;
use std::iter::once;

use flow_sdk::access::GetEventsForBlockIdsRequest;
use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(account: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<(), Box<dyn Error + Send + Sync>> {
    const ERR: &str = "Expected START_HEIGHT and END_HEIGHT or hex encoded block ids";
    let event_ty = args.next().ok_or("Expected event type")?;
    let events = match args.next() {
        Some(s) if s.len() == 64 => {
            let mut block_ids = vec![];
            for data in once(s).chain(args).map(|s| hex::decode(s)) {
                block_ids.push(data?);
            }
            account.client().send(GetEventsForBlockIdsRequest {
                ty: event_ty,
                block_ids,
            }).await?
        }
        Some(s) => {
            let start_height = s.parse()?;
            let end_height = args.next().ok_or(ERR)?.parse()?;
            account.client().events_for_height_range(event_ty, start_height, end_height).await?
        }
        None => bail!("{}", ERR),
    };
    
    for result in events.results {
        println!("\nBlock #{}:", result.block_height);
        for event in result.events {
            println!("{:#?}", event.parse_payload_as_value()?)
        }
    }

    Ok(())
}