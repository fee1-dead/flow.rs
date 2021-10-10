macro_rules! algorithms {
    ($($algo:ident {$($name:ident = $code:expr),+$(,)?})+) => {
        mod private {
            pub trait Sealed {}
        }
        $(
            pub trait $algo: private::Sealed {
                const CODE: u32;
            }
            $(
                pub struct $name;
                impl private::Sealed for $name {}
                impl $algo for $name {
                    const CODE: u32 = $code;
                }
            )+
        )+
    };
}

algorithms! {
    HashAlgorithm {
        Sha2 = 1,
        Sha3 = 3,
    }
    SignatureAlgorithm {
        P256 = 2,
        Secp256k1 = 3,
    }
}

pub trait Signature {
    type Serialized: AsRef<[u8]>;
    fn serialize(&self) -> Self::Serialized;
}

pub trait FlowHasher {
    type Algorithm: HashAlgorithm;
    fn new() -> Self;
    fn update<B: AsRef<[u8]> + ?Sized>(&mut self, bytes: &B);
    fn finalize(self) -> [u8; 32];
}

pub trait FlowSigner {
    type Algorithm: SignatureAlgorithm;
    type SecretKey;
    type PublicKey;
    type Signature: Signature;

    /// Creates a new signer.
    fn new() -> Self;

    /// Creates a signature by consuming a populated hasher and a secret key.
    fn sign(&self, hasher: impl FlowHasher, secret_key: &Self::SecretKey) -> Self::Signature;
    /// Converts a secret key to its public counterpart.
    fn to_public_key(&self, secret_key: &Self::SecretKey) -> Self::PublicKey;
    /// Serializes a public key. Excluding the leading 0x04.
    fn serialize_public_key(&self, public_key: &Self::PublicKey) -> [u8; 64];
}

pub trait SecretKey {
    type Signer: FlowSigner<SecretKey = Self>;
}

#[cfg(feature = "sha3-hash")]
impl FlowHasher for tiny_keccak::Sha3 {
    type Algorithm = Sha3;
    fn new() -> Self {
        tiny_keccak::Sha3::v256()
    }

    fn update<B: AsRef<[u8]> + ?Sized>(&mut self, bytes: &B) {
        use tiny_keccak::Hasher;
        Hasher::update(self, bytes.as_ref())
    }

    fn finalize(self) -> [u8; 32] {
        use tiny_keccak::Hasher;
        let mut output = [0; 32];
        Hasher::finalize(self, &mut output);
        output
    }
}

#[cfg(feature = "secp256k1-sign")]
impl Signature for secp256k1::Signature {
    type Serialized = secp256k1::SerializedSignature;

    fn serialize(&self) -> Self::Serialized {
        self.serialize_der()
    }
}

#[cfg(feature = "secp256k1-sign")]
impl FlowSigner for secp256k1::Secp256k1<secp256k1::SignOnly> {
    type Algorithm = Secp256k1;

    type PublicKey = secp256k1::PublicKey;

    type SecretKey = secp256k1::SecretKey;

    type Signature = secp256k1::Signature;

    fn new() -> Self {
        Self::signing_only()
    }

    fn sign(&self, hasher: impl FlowHasher, secret_key: &Self::SecretKey) -> Self::Signature {
        struct TTBH([u8; 32]);
        impl secp256k1::ThirtyTwoByteHash for TTBH {
            fn into_32(self) -> [u8; 32] {
                self.0
            }
        }
        self.sign(
            &secp256k1::Message::from(TTBH(hasher.finalize())),
            secret_key,
        )
    }

    fn to_public_key(&self, secret_key: &Self::SecretKey) -> Self::PublicKey {
        secp256k1::PublicKey::from_secret_key(self, secret_key)
    }

    fn serialize_public_key(&self, public_key: &Self::PublicKey) -> [u8; 64] {
        let [_, rest @ ..] = public_key.serialize_uncompressed();
        rest
    }
}

#[cfg(feature = "secp256k1-sign")]
impl SecretKey for secp256k1::SecretKey {
    type Signer = secp256k1::Secp256k1<secp256k1::SignOnly>;
}
