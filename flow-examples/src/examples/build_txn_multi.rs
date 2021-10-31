use std::fs;
use std::str::SplitWhitespace;

use anyhow::*;
use cadence_json::ValueOwned;
use flow_sdk::multi::{PartyBuilder, PartyTransaction};
use flow_sdk::prelude::*;
use tokio::sync::Mutex;

use flow_sdk::algorithms::secp256k1::*;
use flow_sdk::algorithms::rand;

use crate::*;

crate::example!(run);

type Txn = PartyTransaction<Box<[u8]>, [u8; 64]>;

pub static BUILT_TXN: Mutex<Option<Txn>> = Mutex::const_new(None);

async fn run(service: &mut ExampleAccount, args: &mut SplitWhitespace<'_>) -> Result<()> {
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

    let mut rng = rand::rngs::EntropyRng::new();
    let secp = Secp256k1::signing_only();

    macro_rules! to_pk {
        ($sk:expr) => {
            PublicKey::from_secret_key(&secp, $sk)
        };
    }

    let acc1sk1 = SecretKey::new(&mut rng);
    let acc1sk2 = SecretKey::new(&mut rng);
    let acc2sk1 = SecretKey::new(&mut rng);
    let acc2sk2 = SecretKey::new(&mut rng);

    let create_acc1 = CreateAccountWeightedTransaction {
        public_keys: &[
            (to_pk!(&acc1sk1), "500".parse()?),
            (to_pk!(&acc1sk2), "500".parse()?),
        ],
    };

    let create_acc2 = CreateAccountWeightedTransaction {
        public_keys: &[
            (to_pk!(&acc2sk1), "500".parse()?),
            (to_pk!(&acc2sk2), "500".parse()?),
        ],
    };

    let acc1_txn = service
        .send_transaction_header(&create_acc1.to_header::<_, DefaultHasher>(&secp))
        .await?
        .finalize(service.client())
        .await?
        .with_context(|| "Timed out while creating account")?;

    let acc2_txn = service
        .send_transaction_header(&create_acc2.to_header::<_, DefaultHasher>(&secp))
        .await?
        .finalize(service.client())
        .await?
        .with_context(|| "Timed out while creating account")?;

    let acc1_addr = acc1_txn
        .events
        .into_iter()
        .find(|e| e.ty == "flow.AccountCreated")
        .with_context(|| "No account created")?
        .parse_payload()?;

    let acc1_addr = acc1_addr
        .find_field("address")
        .expect("No address in account payload")
        .expect_address();

    let acc2_addr = acc2_txn
        .events
        .into_iter()
        .find(|e| e.ty == "flow.AccountCreated")
        .with_context(|| "No account created")?
        .parse_payload()?;

    let acc2_addr = acc2_addr
        .find_field("address")
        .expect("No address in account payload")
        .expect_address();

    let mut acc1 = Account::<_, _>::new_multisign(
        service.client_cloned(),
        &acc1_addr.data,
        0,
        &[acc1sk1, acc1sk2],
    )
    .await?;
    let acc2 = Account::<_, _>::new_multisign(
        service.client_cloned(),
        &acc2_addr.data,
        0,
        &[acc2sk1, acc2sk2],
    )
    .await?;

    let mut party = PartyBuilder::new()
        .script(script)
        .arguments(arguments)
        .latest_block_as_reference(service.client()).await?
        .proposer_account(&mut acc1).await?
        .payer_account(&acc1)
        .authorizer_accounts([&acc1, &acc2])
        .build();

    acc2.sign_party(&mut party);
    
    let txn = acc1.sign_party_as_payer(party);

    let mut lock = BUILT_TXN.lock().await;

    if let Some(prev_txn) = lock.take() {
        println!("Discarding previously built transaction: {:#?}", prev_txn);
    }

    println!("Built: {:#?}", txn);

    *lock = Some(txn);

    Ok(())
}
