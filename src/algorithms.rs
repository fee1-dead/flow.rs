//! The hashing and signing algorithms.

/// The default hasher, the exact type depends on the feature flags enabled.
pub type DefaultHasher = DefaultHasherNoDoc;

/// The default signer, the exact type depends on the feature flags enabled.
pub type DefaultSigner = DefaultSignerNoDoc;

/// The default secret key, the exact type depends on the feature flags enabled.
pub type DefaultSecretKey = DefaultSecretKeyNoDoc;

include!("algorithms/macro_impl.rs");
macro_rules! algorithms {
    ($($tt:tt)+) => {
        algorithms_impl!($($tt)+);
    };
}

algorithms! {
    /// A hashing algorithm.
    HashAlgorithm {
        /// SHA2 256 bit hashing.
        Sha2 = (1, "SHA2_256"),

        /// SHA3 256 bit hashing.
        Sha3 = (3, "SHA3_256"),
    }

    /// A signature algorithm.
    SignatureAlgorithm {
        /// P256 / secp256r1 curve.
        P256 = (2, "ECDSA_P256"),

        /// secp256k1 curve.
        Secp256k1 = (3, "ECDSA_secp256k1"),
    }
}

/// A signature.
pub trait Signature {
    /// Serialized form of the signature
    type Serialized: AsRef<[u8]> + Clone;

    /// Serializes the signature.
    fn serialize(&self) -> Self::Serialized;
}

/// A hasher.
pub trait FlowHasher {
    /// The algorithm of this hasher.
    type Algorithm: HashAlgorithm;

    /// Creates a new hasher.
    fn new() -> Self;

    /// Updates the hasher with bytes.
    fn update<B: AsRef<[u8]> + ?Sized>(&mut self, bytes: &B);

    /// Finalize the hasher, returns the 256 bit hash.
    fn finalize(self) -> [u8; 32];
}

/// A signature signer.
pub trait FlowSigner {
    /// The algorithm of this signer.
    type Algorithm: SignatureAlgorithm;

    /// The secret key used by this signer.
    type SecretKey: Clone;

    /// The public key used by this signer.
    type PublicKey: Copy;

    /// The signature type produced by this signer.
    type Signature: Signature;

    /// Creates a new signer.
    fn new() -> Self;

    /// Creates a signature by consuming a populated hasher and a secret key.
    fn sign(&self, hasher: impl FlowHasher, secret_key: &Self::SecretKey) -> Self::Signature {
        self.sign_populated(hasher.finalize(), secret_key)
    }

    /// Signs a 256 bit hashed data with the secret key.
    fn sign_populated(&self, hashed: [u8; 32], secret_key: &Self::SecretKey) -> Self::Signature;

    /// Converts a secret key to its public counterpart.
    fn to_public_key(&self, secret_key: &Self::SecretKey) -> Self::PublicKey;

    /// Serializes a public key. Excluding the leading 0x04.
    fn serialize_public_key(&self, public_key: &Self::PublicKey) -> [u8; 64];
}

/// A secret key.
pub trait SecretKey {
    /// The signer associated to this secret key.
    type Signer: FlowSigner<SecretKey = Self>;
}

#[cfg(feature = "secp256k1-sign")]
/// Re-exports items from the `secp256k1` crate.
pub mod secp256k1 {
    pub use secp256k1::*;
}

#[cfg(feature = "sha3-hash")]
/// Re-exports items from the `tiny_keccak` crate.
pub mod sha3 {
    pub use tiny_keccak::*;
}

#[cfg(feature = "secp256k1-rand")]
/// Re-exports items from the `rand` crate.
pub mod rand {
    pub use rand::*;
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
    type Serialized = [u8; 64];

    fn serialize(&self) -> Self::Serialized {
        self.serialize_compact()
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

    fn sign_populated(&self, hashed: [u8; 32], secret_key: &Self::SecretKey) -> Self::Signature {
        struct Ttbh([u8; 32]);
        impl secp256k1::ThirtyTwoByteHash for Ttbh {
            fn into_32(self) -> [u8; 32] {
                self.0
            }
        }
        self.sign(&secp256k1::Message::from(Ttbh(hashed)), secret_key)
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

#[cfg(feature = "sha3-hash")]
type DefaultHasherNoDoc = tiny_keccak::Sha3;

#[cfg(feature = "secp256k1-sign")]
type DefaultSignerNoDoc = secp256k1::Secp256k1<secp256k1::SignOnly>;

#[cfg(feature = "secp256k1-sign")]
type DefaultSecretKeyNoDoc = secp256k1::SecretKey;

#[cfg(not(any(feature = "sha3-hash")))]
type DefaultHasherNoDoc = NoDefaultHasherAvailable;

#[cfg(not(any(feature = "sha3-hash")))]
type DefaultSignerNoDoc = NoDefaultSignerAvailable;

#[cfg(not(any(feature = "sha3-hash")))]
#[doc(hidden)]
pub struct NoDefaultHasherAvailable;

#[cfg(not(any(feature = "secp256k1-sign")))]
#[doc(hidden)]
pub struct NoDefaultSignerAvailable;

#[cfg(not(any(feature = "secp256k1-sign")))]
#[doc(hidden)]
pub struct NoDefaultSecretKeyAvailable;
