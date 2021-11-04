use std::error::Error;

use ::cadence_json::AddressOwned;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use crate::access::SendTransactionRequest;
use crate::prelude::*;

const ONEKEY_1_ADDRESS: &str = "0x41c60c9bacab2a3d";
const ONEKEY_1_SK: &str = "74cd94fc21e264811c97bb87f1061edc93aaeedb6885ff8307608a9f2bcebec5";

const ONEKEY_2_ADDRESS: &str = "0x6abc82b79b9a5573";
const ONEKEY_2_SK: &str = "10d5ba77219d1074c8fd7b2a8990e0873e70183e2388300eeb4d332495f5d636";

const MULTISIG_1_ADDRESS: &str = "0x750859bbbd3fe597";
const MULTISIG_1_SK_1: &str = "db8b853c24795cba465b7d70a7ebeb8eed06f1c18307e58885dd54db478f17fd";
const MULTISIG_1_SK_2: &str = "ec4917f95c5d59a7b3967ba67f0a43e2bbf619f3119837429ec6efe05d11ed12";

const MULTISIG_2_ADDRESS: &str = "0x214e531d64c8151a";
const MULTISIG_2_SK_1: &str = "fdf68c79fb7234b15b3cad54e2d6f424e831c7c09dadd277f8cbe27b74a30dcb";
const MULTISIG_2_SK_2: &str = "145f3687501494168f85457f8e7fcd02b8251a5ca10cfe9b73395a7f9aaaee85";

#[tokio::test]
async fn signing_transactions_one_one() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = TonicHyperFlowClient::testnet().await?;

    let secp256k1 = Secp256k1::signing_only();
    let secret_key_raw = hex::decode(ONEKEY_1_SK).unwrap();
    let secret_key = SecretKey::from_slice(&secret_key_raw).unwrap();
    let public_key = PublicKey::from_secret_key(&secp256k1, &secret_key);

    let txn = CreateAccountTransaction {
        public_keys: &[public_key],
    };
    let txn = txn.to_header::<_, DefaultHasher>(&secp256k1);

    let address: AddressOwned = ONEKEY_1_ADDRESS.parse().unwrap();

    let mut account = Account::<_, _>::new(client, &address.data, secret_key).await?;

    let latest_block = account.client().latest_block_header(Seal::Sealed).await?.id;
    let sequence_number = account.primary_key_sequence_number().await?;

    account.sign_transaction_header(&txn, latest_block, sequence_number as u64, 1000);

    Ok(())
}

#[tokio::test]
async fn signing_transactions_multisig_one() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = TonicHyperFlowClient::testnet().await?;

    let secp256k1 = Secp256k1::signing_only();
    let sk1 = hex::decode(MULTISIG_1_SK_1).unwrap();
    let sk2 = hex::decode(MULTISIG_1_SK_2).unwrap();
    let sk1 = SecretKey::from_slice(&sk1).unwrap();
    let sk2 = SecretKey::from_slice(&sk2).unwrap();

    let pk1 = PublicKey::from_secret_key(&secp256k1, &sk1);

    let txn = CreateAccountTransaction {
        public_keys: &[pk1],
    };
    let txn = txn.to_header::<_, DefaultHasher>(&secp256k1);

    let address: AddressOwned = MULTISIG_1_ADDRESS.parse().unwrap();

    let mut account = Account::<_, _>::new_multisign(client, &address.data, 0, &[sk1, sk2]).await?;

    let latest_block = account.client().latest_block_header(Seal::Sealed).await?.id;
    let sequence_number = account.primary_key_sequence_number().await?;

    account.sign_transaction_header(&txn, latest_block, sequence_number as u64, 1000);

    Ok(())
}

#[tokio::test]
async fn signing_transactions_one_multi() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = TonicHyperFlowClient::testnet().await?;
    let client2 = client.clone();

    let secp256k1 = Secp256k1::signing_only();
    let sk1 = hex::decode(ONEKEY_1_SK).unwrap();
    let sk1 = SecretKey::from_slice(&sk1).unwrap();
    let sk2 = hex::decode(ONEKEY_2_SK).unwrap();
    let sk2 = SecretKey::from_slice(&sk2).unwrap();
    let pk = PublicKey::from_secret_key(&secp256k1, &sk1);
    let address1: AddressOwned = ONEKEY_1_ADDRESS.parse().unwrap();
    let address2: AddressOwned = ONEKEY_2_ADDRESS.parse().unwrap();

    let txn = CreateAccountTransaction { public_keys: &[pk] };
    let txn = txn.to_header::<_, DefaultHasher>(&secp256k1);

    let mut account1 = Account::<_, _>::new(client, &address1.data, sk1).await?;
    let account2 = Account::<_, _>::new(client2, &address2.data, sk2).await?;

    let mut party = txn
        .into_party_builder()
        .authorizer_account(&account1)
        .proposer_account(&mut account1)
        .await?
        .payer_account(&account2)
        .latest_block_as_reference(account1.client())
        .await?
        .build_prehashed::<DefaultHasher>();

    account1.sign_party(&mut party);

    let txn = account2.sign_party_as_payer(party);

    println!("{:?}", txn);

    if false {
        account1.client().send_transaction(txn).await?;
    }

    Ok(())
}

#[tokio::test]
async fn signing_transactions_one_multi_authorizers() -> Result<(), Box<dyn Error + Send + Sync>> {
    const SCRIPT: &str = "
    transaction {
        prepare(acct1: AuthAccount, acct2: AuthAccount) {
            log([acct1, acct2])
        }
    }";
    let client = TonicHyperFlowClient::testnet().await?;
    let client2 = client.clone();

    let sk1 = hex::decode(ONEKEY_1_SK).unwrap();
    let sk1 = SecretKey::from_slice(&sk1).unwrap();
    let sk2 = hex::decode(ONEKEY_2_SK).unwrap();
    let sk2 = SecretKey::from_slice(&sk2).unwrap();
    let address1: AddressOwned = ONEKEY_1_ADDRESS.parse().unwrap();
    let address2: AddressOwned = ONEKEY_2_ADDRESS.parse().unwrap();

    let txn = TransactionHeaderBuilder::new()
        .script_static(SCRIPT)
        .build();

    let mut account1 = Account::<_, _>::new(client, &address1.data, sk1).await?;
    let account2 = Account::<_, _>::new(client2, &address2.data, sk2).await?;

    let mut party = txn
        .into_party_builder()
        .authorizer_accounts([&account1, &account2])
        .proposer_account(&mut account1)
        .await?
        .payer_account(&account2)
        .latest_block_as_reference(account1.client())
        .await?
        .build_prehashed::<DefaultHasher>();

    account1.sign_party(&mut party);

    let txn = account2.sign_party_as_payer(party);

    println!("{:?}", txn);

    if false {
        account1.client().send_transaction(txn).await?;
    }

    Ok(())
}

#[tokio::test]
async fn signing_transactions_multisig_multi() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = TonicHyperFlowClient::testnet().await?;
    let client2 = client.clone();

    let secp = Secp256k1::signing_only();
    let sk1_1 = hex::decode(MULTISIG_1_SK_1).unwrap();
    let sk1_1 = SecretKey::from_slice(&sk1_1).unwrap();
    let sk1_2 = hex::decode(MULTISIG_1_SK_2).unwrap();
    let sk1_2 = SecretKey::from_slice(&sk1_2).unwrap();
    let sk2_1 = hex::decode(MULTISIG_2_SK_1).unwrap();
    let sk2_1 = SecretKey::from_slice(&sk2_1).unwrap();
    let sk2_2 = hex::decode(MULTISIG_2_SK_2).unwrap();
    let sk2_2 = SecretKey::from_slice(&sk2_2).unwrap();
    let pk = PublicKey::from_secret_key(&secp, &sk1_1);
    let address1: AddressOwned = MULTISIG_1_ADDRESS.parse().unwrap();
    let address2: AddressOwned = MULTISIG_2_ADDRESS.parse().unwrap();

    let txn = CreateAccountTransaction { public_keys: &[pk] };

    let txn = txn.to_header::<_, DefaultHasher>(&secp);

    let mut account1 =
        Account::<_, _>::new_multisign(client, &address1.data, 0, &[sk1_1, sk1_2]).await?;
    let account2 =
        Account::<_, _>::new_multisign(client2, &address2.data, 0, &[sk2_1, sk2_2]).await?;

    let mut party = txn
        .into_party_builder()
        .authorizer_account(&account1)
        .proposer_account(&mut account1)
        .await?
        .payer_account(&account2)
        .latest_block_as_reference(account1.client())
        .await?
        .build_prehashed::<DefaultHasher>();

    account1.sign_party(&mut party);

    let txn = account2.sign_party_as_payer(party);

    println!("{:?}", txn);

    if false {
        account1
            .client()
            .send(SendTransactionRequest { transaction: txn })
            .await?;
    }

    Ok(())
}

// #[tokio::test]
async fn _create_accounts() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = TonicHyperFlowClient::testnet().await?;

    let secp256k1 = Secp256k1::signing_only();
    let my_secret_key = SecretKey::from_slice(&hex::decode(ONEKEY_1_SK).unwrap()).unwrap();
    let sk1 = hex::decode(MULTISIG_2_SK_1).unwrap();
    let sk2 = hex::decode(MULTISIG_2_SK_2).unwrap();
    let sk1 = SecretKey::from_slice(&sk1).unwrap();
    let sk2 = SecretKey::from_slice(&sk2).unwrap();
    let pk1 = PublicKey::from_secret_key(&secp256k1, &sk1);
    let pk2 = PublicKey::from_secret_key(&secp256k1, &sk2);

    let txn = CreateAccountWeightedTransaction {
        public_keys: &[(pk1, "500".parse().unwrap()), (pk2, "500".parse().unwrap())],
    };
    let txn = txn.to_header::<_, DefaultHasher>(&secp256k1);

    let address: AddressOwned = ONEKEY_1_ADDRESS.parse().unwrap();

    let mut account = Account::<_, _>::new(client, &address.data, my_secret_key).await?;

    let res = account.send_transaction_header(&txn).await?;

    println!("{}", hex::encode(&res.id));

    let fin = res.finalize(account.client()).await?.unwrap();

    for event in fin.events {
        if event.ty == "flow.AccountCreated" {
            println!("{:?}", event.parse_payload().unwrap());
        }
    }

    Ok(())
}
