use std::error::Error;

use cadence_json::ValueRef;
use secp256k1::SecretKey;

use crate::client::TonicHyperFlowClient;
use crate::sign::{One, SignMethod};
use crate::transaction::TransactionHeaderBuilder;

const SCRIPT: &str = r#"
    transaction(greeting: String) {
       let guest: Address

       prepare(authorizer: AuthAccount) {
           self.guest = authorizer.address
       }

       execute {
           log(greeting.concat(",").concat(guest.toString()))
       }
    }
"#;

#[tokio::test]
async fn building_transaction_headers() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = TonicHyperFlowClient::testnet()?;
    let argument = ValueRef::String("Hello");

    let header = TransactionHeaderBuilder::new()
        .script_static(SCRIPT)
        .argument(&argument)
        .build();
    let secret_key = SecretKey::new(&mut rand::thread_rng());

    // Make a dummy account to ensure that we can sign the header.
    let mut acc = unsafe {
        crate::account::Account::<_, _>::new_unchecked(
            client,
            Default::default(),
            SignMethod::One(One {
                key_id: 1,
                key: secret_key,
            }),
        )
    };

    acc.sign_transaction_header(&header, "", 0, 0);

    if false {
        // cannot actually be run. Confirm that this compiles.
        acc.send_transaction_header(&header).await?;
    }

    Ok(())
}
