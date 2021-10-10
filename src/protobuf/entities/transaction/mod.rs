mod signing;
pub use signing::*;

use otopr::*;

pub type RepSlice<'a, T> = Repeated<T, &'a [T]>;

#[derive(Enumeration, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Unknown = 0,
    Pending = 1,
    Finalized = 2,
    Executed = 3,
    Sealed = 4,
    Expired = 5,
}

#[derive(EncodableMessage, Clone, Copy, PartialEq, Eq)]
pub struct ProposalKeyE<'a> {
    pub address: &'a [u8],
    pub key_id: u32,
    pub sequence_number: u64,
}

#[derive(DecodableMessage, Default)]
pub struct ProposalKeyD {
    pub address: Vec<u8>,
    pub key_id: u32,
    pub sequence_number: u32,
}

#[derive(EncodableMessage, Clone, Copy, PartialEq, Eq)]
pub struct SignatureE<'a> {
    pub address: &'a [u8],
    pub key_id: u32,
    pub signature: &'a [u8],
}

#[derive(DecodableMessage, Default)]
pub struct SignatureD {
    pub address: Vec<u8>,
    pub key_id: u32,
    pub signature: Vec<u8>,
}

#[derive(EncodableMessage, Clone, Copy, PartialEq, Eq)]
pub struct TransactionE<'a> {
    pub script: &'a [u8],
    pub arguments: RepSlice<'a, &'a [u8]>,
    pub reference_block_id: &'a [u8],
    pub gas_limit: u64,
    pub proposal_key: ProposalKeyE<'a>,
    pub payer: &'a [u8],
    pub authorizers: RepSlice<'a, &'a [u8]>,
    pub payload_signatures: RepSlice<'a, SignatureE<'a>>,
    pub envelope_signatures: RepSlice<'a, SignatureE<'a>>,
}

#[derive(DecodableMessage, Default)]
pub struct TransactionD {
    pub script: Vec<u8>,
    pub arguments: Repeated<Vec<u8>>,
    pub reference_block_id: Vec<u8>,
    pub gas_limit: u64,
    pub proposal_key: ProposalKeyD,
    pub payer: Vec<u8>,
    pub authorizers: Repeated<Vec<u8>>,
    pub payload_signatures: Repeated<SignatureD>,
    pub envelope_signatures: Repeated<SignatureD>,
}
