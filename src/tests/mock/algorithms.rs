use crate::algorithms::*;

pub struct MockSigner;

pub struct MockHasher([u8; 32]);

#[derive(PartialEq, Eq, Debug)]
pub struct MockSig(pub [u8; 32]);

pub type MockKey = [u8; 64];

impl Signature for MockSig {
    type Serialized = [u8; 64];

    fn serialize(&self) -> Self::Serialized {
        [0; 64]
    }
}

impl FlowSigner for MockSigner {
    type Algorithm = Secp256k1;

    type SecretKey = [u8; 64];

    type PublicKey = [u8; 64];

    type Signature = MockSig;

    fn new() -> Self {
        Self
    }

    fn sign_populated(&self, hashed: [u8; 32], _: &Self::SecretKey) -> MockSig {
        MockSig(hashed)
    }

    fn to_public_key(&self, sk: &Self::SecretKey) -> [u8; 64] {
        *sk
    }

    fn serialize_public_key(&self, pk: &Self::PublicKey) -> [u8; 64] {
        *pk
    }
}

impl FlowHasher for MockHasher {
    type Algorithm = Sha3;

    fn new() -> Self {
        Self([0; 32])
    }

    fn update<B: AsRef<[u8]> + ?Sized>(&mut self, data: &B) {
        let data = data.as_ref();
        self.0[0..data.len()].copy_from_slice(data);
    }

    fn finalize(self) -> [u8; 32] {
        self.0
    }
}
