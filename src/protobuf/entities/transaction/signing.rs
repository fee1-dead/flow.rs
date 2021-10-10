use std::collections::HashSet;

use otopr::Repeated;
use rlp::RlpStream;

use crate::{
    algorithms::{FlowHasher, FlowSigner, SecretKey},
    ProposalKeyE, RepSlice, SignatureE, TransactionE,
};

/// All the data one needs to sign a transaction payload.
///
/// Sign this only if you are the proposer or one of the authorizers of the transaction
pub struct TransactionPayload<'a> {
    pub script: &'a [u8],
    pub arguments: &'a [&'a [u8]],
    pub reference_block_id: &'a [u8],
    pub gas_limit: u64,
    pub proposal_key_address: &'a [u8],
    pub proposal_key_id: u32,
    pub proposal_key_sequence_number: u64,
    pub payer: &'a [u8],
    pub authorizers: &'a [&'a [u8]],
}

pub struct PayloadSignature<S> {
    /// The **canonical index** of the signer.
    pub signer_index: u32,
    pub key_id: u32,
    pub signature: S,
}

pub struct TransactionEnvelope<'a, S> {
    pub payload: TransactionPayload<'a>,
    pub payload_signatures: &'a [PayloadSignature<S>],
}

const fn padded<const N: usize>(src: &[u8]) -> [u8; N] {
    let mut new_buf = [0; N];

    let mut i = 0;

    while i < src.len() {
        new_buf[i] = src[i];
        i += 1;
    }

    new_buf
}

const PADDED_LEN: usize = 32;
const PADDED_TRANSACTION_DOMAIN_TAG: [u8; PADDED_LEN] =
    padded::<PADDED_LEN>(b"FLOW-V0.0-transaction");

/// common methods for payload and envelope.
macro_rules! sign_methods {
    () => {
        /// Creates a signature using a secret key.
        #[cfg(any(feature = "sha3-hash"))]
        pub fn sign<T: SecretKey>(&self, secret_key: &T) -> <T::Signer as FlowSigner>::Signature {
            let signer = T::Signer::new();
            self.sign_with(secret_key, &signer)
        }

        /// Creates a signature using a secret key, a signer, and the default hasher.
        #[cfg(any(feature = "sha3-hash"))]
        pub fn sign_with<T: SecretKey<Signer = S>, S: FlowSigner<SecretKey = T>>(
            &self,
            secret_key: &T,
            signer: &S,
        ) -> S::Signature {
            self.sign_custom(secret_key, signer, tiny_keccak::Sha3::v256())
        }

        /// Creates a signature using a secret key, a signer, and a hasher.
        pub fn sign_custom<
            T: SecretKey<Signer = S>,
            S: FlowSigner<SecretKey = T>,
            H: FlowHasher,
        >(
            &self,
            secret_key: &T,
            signer: &S,
            mut hasher: H,
        ) -> S::Signature {
            let mut stream = RlpStream::new();
            self.rlp_encode(&mut stream);
            hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
            hasher.update(&stream.out());
            signer.sign(hasher, secret_key)
        }
    };
}

impl<'a> TransactionPayload<'a> {
    sign_methods!();

    pub fn to_transaction(
        self,
        payload_signatures: RepSlice<'a, SignatureE<'a>>,
        envelope_signatures: RepSlice<'a, SignatureE<'a>>,
    ) -> TransactionE<'a> {
        let TransactionPayload {
            script,
            arguments,
            reference_block_id,
            gas_limit,
            proposal_key_address,
            proposal_key_id,
            proposal_key_sequence_number,
            payer,
            authorizers,
        } = self;

        TransactionE {
            script,
            arguments: Repeated::new(arguments),
            reference_block_id,
            gas_limit,
            proposal_key: ProposalKeyE {
                address: proposal_key_address,
                key_id: proposal_key_id,
                sequence_number: proposal_key_sequence_number,
            },
            payer,
            authorizers: Repeated::new(authorizers),
            payload_signatures,
            envelope_signatures,
        }
    }

    /// Build a list with all the signers of this transaction. Indices are **canonical**.
    pub fn signers(&self) -> Vec<&'a [u8]> {
        let mut seen = HashSet::new();
        let mut signers = Vec::with_capacity(self.authorizers.len());

        let mut add_signer = |addr: &'a [u8]| {
            if seen.insert(addr) {
                signers.push(addr)
            }
        };

        if !self.proposal_key_address.iter().all(|n| *n == 0) {
            add_signer(self.proposal_key_address);
        }

        if !self.payer.iter().all(|n| *n == 0) {
            add_signer(self.payer);
        }

        for authorizer in self.authorizers {
            add_signer(authorizer);
        }

        signers
    }

    /// Encodes to an RLP buffer with the payload. The field order must not be changed.
    fn rlp_encode(&self, stream: &mut RlpStream) {
        let TransactionPayload {
            script,
            arguments,
            reference_block_id,
            gas_limit,
            proposal_key_address,
            proposal_key_id,
            proposal_key_sequence_number,
            payer,
            authorizers,
        } = self;

        stream
            .begin_list(9)
            .append(script)
            .append_list::<&[u8], _>(arguments)
            .append(reference_block_id)
            .append(gas_limit)
            .append(proposal_key_address)
            .append(proposal_key_id)
            .append(proposal_key_sequence_number)
            .append(payer)
            .append_list::<&[u8], _>(authorizers);
    }
}

impl<'a, B: AsRef<[u8]>> TransactionEnvelope<'a, B> {
    sign_methods!();

    fn rlp_encode(&self, stream: &mut RlpStream) {
        stream.begin_list(2);

        self.payload.rlp_encode(stream);

        stream.begin_list(self.payload_signatures.len());

        for PayloadSignature {
            signer_index,
            key_id,
            signature,
        } in self.payload_signatures
        {
            stream
                .begin_list(3)
                .append(signer_index)
                .append(key_id)
                .append(&signature.as_ref());
        }
    }
}
