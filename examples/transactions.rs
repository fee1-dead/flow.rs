use std::{
    error::Error,
    io::{stdin, BufRead},
};

use cadence_json::{AddressOwned, ValueOwned};
use flow_sdk::{access::SimpleAccount, client::TonicHyperFlowClient, CreateAccountTransaction};

use secp256k1::{PublicKey, Secp256k1, SecretKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();

    // Let's make a transaction!

    // First, generate a keypair for us to use.
    let secp = Secp256k1::signing_only();
    let mut rng = secp256k1::rand::thread_rng();
    let secret_key = SecretKey::new(&mut rng);
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    // Print the public key as a hexadecimal.
    println!("This is your public key:");
    let public_key_bytes = public_key.serialize_uncompressed();
    // There is a leading 0x04 which we don't need to emit
    // https://bitcoin.stackexchange.com/a/3043
    for ch in &public_key_bytes[1..] {
        print!("{:02x}", *ch);
    }
    println!();
    println!("Go to https://flow-faucet.vercel.app/, select secp256k1 and sha3 and create your testnet account.");
    println!("Paste the address you get and press ENTER to continue");

    stdin.read_line(&mut buf)?;

    let addr = buf.trim();

    let address: AddressOwned = addr.parse()?;
    let net = TonicHyperFlowClient::testnet()?;

    let mut account =
        SimpleAccount::<_, _>::new(net.into_inner(), &address.data, secret_key).await?;

    let create_account = CreateAccountTransaction {
        public_keys: &[account.public_key()],
    };

    let create_account_header = create_account.to_header::<_, tiny_keccak::Sha3>(account.signer());
    let res = account
        .send_transaction_header(&create_account_header)
        .await?;

    println!(
        "Just made {} to create another account :p",
        hex::encode(&res.id)
    );

    let response = res.finalize(account.client()).await?;

    match response {
        Some(res) => {
            for ev in res.events.iter() {
                if ev.ty == "flow.AccountCreated" {
                    let payload = ev.parse_payload()?;
                    match payload {
                        ValueOwned::Event(composite) => {
                            let addr = composite
                                .fields
                                .into_iter()
                                .find(|field| field.name == "address")
                                .map(|field| field.value)
                                .unwrap();
                            match addr {
                                ValueOwned::Address(addr) => {
                                    println!("Created account's address is: {}", addr);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None => {
            panic!("The transaction did not get sealed... Perhaps the network is malfunctioning?")
        }
    }

    Ok(())
}
