use std::error::Error;
use std::str::SplitWhitespace;

use flow_sdk::prelude::*;
use flow_sdk::transaction::CreateAccountTransaction;

use crate::ExampleAccount;

crate::example!(run);

async fn run(
    account: &mut ExampleAccount,
    _: &mut SplitWhitespace<'_>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let signer = flow_sdk::algorithms::secp256k1::Secp256k1::signing_only();

    let txn = CreateAccountTransaction {
        public_keys: &[account.primary_public_key()],
    };

    let txn = txn.to_header::<_, DefaultHasher>(&signer);

    let txn = account.send_transaction_header(&txn).await?;

    let res = txn.finalize(account.client()).await?.unwrap();

    for event in res.events {
        if event.ty == "flow.AccountCreated" {
            let payload = event.parse_payload()?;
            let address = payload.find_field("address").unwrap().expect_address();
            println!("Created {}", address);
        }
    }

    Ok(())
}
