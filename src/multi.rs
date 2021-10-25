use std::ops::Deref;

use rlp::RlpStream;

use crate::account::PADDED_TRANSACTION_DOMAIN_TAG;
use crate::algorithms::FlowHasher;
use crate::transaction::rlp_encode_transaction_envelope;
use crate::transaction::rlp_encode_transaction_payload;

pub trait Party<H: FlowHasher> {
    fn payload(&self) -> H;
    fn add_payload_signature(&mut self, signer_index: usize, signer_id: u32, signature: [u8; 64]);
    fn envelope(&self) -> H;
}

/// A basic signing party. Contains all the information needed to make signatures.
#[derive(Clone)]
pub struct SigningParty {
    script: Box<str>,
    arguments: Box<[Box<[u8]>]>,
    reference_block: Box<[u8]>,
    gas_limit: u64,
    proposer_address: Box<[u8]>,
    proposal_key_id: u64,
    proposal_key_sequence_number: u64,
    payer: Box<[u8]>,
    authorizers: Box<[Box<[u8]>]>,
    payload_signatures: Vec<(usize, u32, [u8; 64])>,
}

impl<H: FlowHasher> Party<H> for SigningParty {
    fn payload(&self) -> H {
        let mut hasher = H::new();
        hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
        let mut stream = RlpStream::new();
        rlp_encode_transaction_payload(
            &mut stream,
            &*self.script,
            self.arguments.iter(),
            self.reference_block.iter(),
            self.gas_limit,
            &self.proposer_address,
            self.proposal_key_id,
            self.proposal_key_sequence_number,
            &self.payer,
            self.authorizers.iter(),
        );
        hasher.update(&stream.out());

        hasher
    }

    fn add_payload_signature(&mut self, signer_index: usize, signer_id: u32, signature: [u8; 64]) {
        self.payload_signatures
            .push((signer_index, signer_id, signature));
    }

    fn envelope(&self) -> H {
        let mut hasher = H::new();
        hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
        let mut stream = RlpStream::new();
        rlp_encode_transaction_envelope(
            &mut stream,
            &*self.script,
            self.arguments.iter(),
            &self.reference_block,
            self.gas_limit,
            &self.proposer_address,
            self.proposal_key_id,
            self.proposal_key_sequence_number,
            &self.payer,
            self.authorizers.iter(),
            self.payload_signatures
                .iter()
                .copied()
                .map(|(idx, key_id, sig)| (&self.authorizers[idx], key_id, sig)),
        );
        hasher.update(&stream.out());
        hasher
    }
}

impl SigningParty {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        script: Box<str>,
        arguments: Box<[Box<[u8]>]>,
        reference_block: Box<[u8]>,
        gas_limit: u64,
        proposer_address: Box<[u8]>,
        proposal_key_id: u64,
        proposal_key_sequence_number: u64,
        payer: Box<[u8]>,
        authorizers: Box<[Box<[u8]>]>,
    ) -> Self {
        Self {
            script,
            arguments,
            reference_block,
            gas_limit,
            proposer_address,
            proposal_key_id,
            proposal_key_sequence_number,
            payer,
            authorizers,
            payload_signatures: Vec::new(),
        }
    }

    pub fn script(&self) -> &str {
        &self.script
    }

    pub fn arguments(&self) -> &[Box<[u8]>] {
        &self.arguments
    }

    pub fn reference_block(&self) -> &[u8] {
        &self.reference_block
    }

    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    pub fn proposer_address(&self) -> &[u8] {
        &self.proposer_address
    }

    pub fn proposer_key_id(&self) -> u64 {
        self.proposal_key_id
    }

    pub fn proposer_sequence_number(&self) -> u64 {
        self.proposal_key_sequence_number
    }

    pub fn payer(&self) -> &[u8] {
        &self.payer
    }

    pub fn authorizers(&self) -> &[Box<[u8]>] {
        &self.authorizers
    }
}

/// A party that prepopulates hashed payload and sends that around for signing.
///
/// The party only supports one type of hashing algorithm,
/// which means that all entities involved must use the same algorithm.
#[derive(Clone)]
pub struct PreHashedParty<H> {
    party: SigningParty,
    payload: H,
}

impl<H: FlowHasher> PreHashedParty<H> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        script: Box<str>,
        arguments: Box<[Box<[u8]>]>,
        reference_block: Box<[u8]>,
        gas_limit: u64,
        proposer_address: Box<[u8]>,
        proposal_key_id: u64,
        proposal_key_sequence_number: u64,
        payer: Box<[u8]>,
        authorizers: Box<[Box<[u8]>]>,
    ) -> Self {
        let party = SigningParty::new(
            script,
            arguments,
            reference_block,
            gas_limit,
            proposer_address,
            proposal_key_id,
            proposal_key_sequence_number,
            payer,
            authorizers,
        );
        let payload = party.payload();
        Self { party, payload }
    }
}

impl<H: FlowHasher + Clone> Party<H> for PreHashedParty<H> {
    fn payload(&self) -> H {
        self.payload.clone()
    }

    fn add_payload_signature(&mut self, signer_index: usize, signer_id: u32, signature: [u8; 64]) {
        <SigningParty as Party<H>>::add_payload_signature(
            &mut self.party,
            signer_index,
            signer_id,
            signature,
        )
    }

    fn envelope(&self) -> H {
        self.party.envelope()
    }
}

impl<H> Deref for PreHashedParty<H> {
    type Target = SigningParty;

    fn deref(&self) -> &SigningParty {
        &self.party
    }
}
