use std::iter::once;
use std::str::SplitWhitespace;

use anyhow::*;
use flow_sdk::access::GetEventsForBlockIdsRequest;
use flow_sdk::prelude::*;

use crate::*;

crate::example!(run);

async fn run(account: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<()> {
    const ERR: &str = "Expected START_HEIGHT and END_HEIGHT or hex encoded block ids";
    let event_ty = args.next().with_context(|| "Expected event type")?;
    let events = match args.next() {
        Some(s) if s.len() == 64 => {
            let mut block_ids = vec![];
            for data in once(s).chain(args).map(hex::decode) {
                block_ids.push(data?);
            }
            account
                .client()
                .send(GetEventsForBlockIdsRequest {
                    ty: event_ty,
                    block_ids,
                })
                .await?
        }
        Some(s) => {
            let start_height = s.parse()?;
            let end_height = args.next().with_context(|| ERR)?.parse()?;
            account
                .client()
                .events_for_height_range(event_ty, start_height, end_height)
                .await?
        }
        None => bail!(ERR),
    };

    for result in events.results {
        println!("\nBlock #{}:", result.block_height);
        for event in result.events {
            println!("{:#?}", event.parse_payload_as_value()?)
        }
    }

    Ok(())
}
