use std::{
    error::Error,
    io::{stdin, BufRead},
};

use flow_sdk::client::TonicHyperFlowClient;

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

    // The default formatter prints the public key as a hexadecimal.
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

    let mut net = TonicHyperFlowClient::testnet()?;

    Ok(())
}
