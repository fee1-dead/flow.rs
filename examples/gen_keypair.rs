use secp256k1::rand::rngs::EntropyRng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

fn main() {
    let secp = Secp256k1::signing_only();

    // `EntropyRng` is a secure random number generator.
    let mut rng = EntropyRng::new();
    let secret_key = SecretKey::new(&mut rng);
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    println!("Secret key: {}", hex::encode(secret_key.as_ref()));

    // https://bitcoin.stackexchange.com/a/3043
    //
    // Flow faucet only accepts hex encoded public key of length 128
    // which means the leading byte of 0x04 must be discarded.
    println!(
        "Public key: {}",
        hex::encode(&public_key.serialize_uncompressed()[1..])
    )
}
