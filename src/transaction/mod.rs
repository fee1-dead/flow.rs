//! Transactions on the flow blockchain.
//!
//! A transaction represents a unit of computation that is submitted to the Flow network.

use std::slice;

use cadence_json::ValueOwned;
use otopr::encoding::EncodeAsRef;
use otopr::*;
use wire_types::*;

pub mod rlp;

mod signing;
pub use signing::*;

mod template;
pub use template::*;

mod finalize;
pub use finalize::*;

use crate::trait_hack::Hack;

/// Status of a transaction.
#[derive(Enumeration, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// The transaction status is not known.
    Unknown = 0,

    /// The transaction has been received by a collector but not yet finalized in a block.
    Pending = 1,

    /// The consensus nodes have finalized the block that the transaction is included in.
    Finalized = 2,

    /// The execution nodes have produced a result for the transaction.
    Executed = 3,

    /// The verification nodes have verified the transaction (the block in which the transaction is)
    /// and the seal is included in the latest block.
    Sealed = 4,

    /// The transaction was submitted past its expiration block height.
    Expired = 5,
}

#[derive(EncodableMessage, Clone, Copy, PartialEq, Eq)]
#[otopr(encode_where_clause(
    where
        Address: AsRef<[u8]>,
))]
/// The proposal key is used to specify a sequence number for the transaction.
///
/// Use this type when sending to the network.
pub struct ProposalKeyE<Address> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// Address of proposer account
    pub address: Address,
    /// ID of proposal key on the proposal account
    pub key_id: u32,
    /// [Sequence number](https://docs.onflow.org/concepts/accounts-and-keys#sequence-numbers) for the proposal key
    pub sequence_number: u64,
}

#[derive(DecodableMessage, Default)]
/// The proposal key is used to specify a sequence number for the transaction.
///
/// This type is used when decoding messages from the network.
pub struct ProposalKeyD {
    /// Address of proposer account
    pub address: Box<[u8]>,
    /// ID of proposal key on the proposal account
    pub key_id: u32,
    /// [Sequence number](https://docs.onflow.org/concepts/accounts-and-keys#sequence-numbers) for the proposal key
    pub sequence_number: u64,
}

#[derive(EncodableMessage, Clone, Copy, PartialEq, Eq)]
#[otopr(encode_where_clause(
    where
        Address: AsRef<[u8]>,
        Signature: AsRef<[u8]>,
))]
/// Signature of a transaction.
///
/// Use this type when sending this over the network.
pub struct SignatureE<Address, Signature> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// Address of the account that signed.
    pub address: Address,

    /// The key id number of the key of the account that signed.
    pub key_id: u32,

    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// The signature.
    pub signature: Signature,
}

/// Signature of a transaction.
///
/// This type is used when decoding messages from a network.
#[derive(DecodableMessage, Default)]
pub struct SignatureD {
    /// Address of the account that signed.
    pub address: Box<[u8]>,

    /// The key id number of the key of the account that signed.
    pub key_id: u32,

    /// The signature.
    pub signature: Box<[u8]>,
}

#[derive(EncodableMessage, Clone, PartialEq, Eq)]
#[otopr(encode_extra_type_params(
    PayloadSignatureAddress,
    PayloadSignature,
    EnvelopeSignatureAddress,
    EnvelopeSignature,
))]
#[otopr(encode_where_clause(
    where
        Script: AsRef<[u8]>,
        ReferenceBlockId: AsRef<[u8]>,
        Payer: AsRef<[u8]>,
        ProposalKeyAddress: AsRef<[u8]>,
        PayloadSignatureAddress: AsRef<[u8]>,
        PayloadSignature: AsRef<[u8]>,
        EnvelopeSignatureAddress: AsRef<[u8]>,
        EnvelopeSignature: AsRef<[u8]>,
        Arguments: HasItem,
        <Arguments as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Arguments: IntoIterator<Item = &'a <Arguments as HasItem>::Item>,
        for<'a> Hack<<&'a Arguments as IntoIterator>::IntoIter>: Clone,
        Authorizers: HasItem,
        <Authorizers as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Authorizers: IntoIterator<Item = &'a <Authorizers as HasItem>::Item>,
        for<'a> Hack<<&'a Authorizers as IntoIterator>::IntoIter>: Clone,
        PayloadSignatures: HasItem<Item = SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> &'a PayloadSignatures: IntoIterator<Item = &'a SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> Hack<<&'a PayloadSignatures as IntoIterator>::IntoIter>: Clone,
        EnvelopeSignatures: HasItem<Item = SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> &'a EnvelopeSignatures: IntoIterator<Item = &'a SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> Hack<<&'a EnvelopeSignatures as IntoIterator>::IntoIter>: Clone,
))]
/// A transaction represents a unit of computation that is submitted to the Flow network.
///
/// Use this type when sending it over the network.
pub struct TransactionE<
    Script,
    Arguments,
    ReferenceBlockId,
    ProposalKeyAddress,
    Payer,
    Authorizers,
    PayloadSignatures,
    EnvelopeSignatures,
> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// Raw source code for a Cadence script, encoded as UTF-8 bytes
    pub script: Script,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |it| it.0.map(EncodeAsRef::new))))]
    /// Arguments passed to the Cadence script, encoded as JSON-Cadence bytes
    pub arguments: Arguments,

    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// Block ID used to determine transaction expiry
    pub reference_block_id: ReferenceBlockId,

    /// The gas limit.
    pub gas_limit: u64,

    /// Account key used to propose the transaction
    pub proposal_key: ProposalKeyE<ProposalKeyAddress>,

    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// Address of the payer account
    pub payer: Payer,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |it| it.0.map(AsRef::as_ref))))]
    /// Addresses of the transaction authorizers
    pub authorizers: Authorizers,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |it| it.0)))]
    /// Signatures from all payload signer accounts
    pub payload_signatures: PayloadSignatures,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |it| it.0)))]
    /// Signatures from all envelope signer accounts
    pub envelope_signatures: EnvelopeSignatures,
}

#[derive(DecodableMessage, Default)]
/// A transaction represents a unit of computation that is submitted to the Flow network.
///
/// This type is used when decoding messages from the network.
pub struct TransactionD {
    /// Raw source code for a Cadence script, encoded as UTF-8 bytes
    pub script: Box<[u8]>,

    /// Arguments passed to the Cadence script, encoded as JSON-Cadence bytes
    pub arguments: Repeated<Vec<Box<[u8]>>>,

    /// Block ID used to determine transaction expiry
    pub reference_block_id: Box<[u8]>,

    /// The gas limit.
    pub gas_limit: u64,

    /// Account key used to propose the transaction
    pub proposal_key: ProposalKeyD,

    /// Address of the payer account
    pub payer: Box<[u8]>,

    /// Addresses of the transaction authorizers
    pub authorizers: Repeated<Vec<Box<[u8]>>>,

    /// Signatures from all payload signer accounts
    pub payload_signatures: Repeated<Vec<SignatureD>>,

    /// Signatures from all envelope signer accounts
    pub envelope_signatures: Repeated<Vec<SignatureD>>,
}

impl TransactionD {
    /// Parse a specific argument.
    pub fn parse_argument(&self, index: usize) -> serde_json::Result<ValueOwned> {
        let arg = &self.arguments[index];

        serde_json::from_slice(arg)
    }

    /// Returns an iterator parsing the underlying arguments.
    pub fn parse_arguments(&self) -> ParseArguments<slice::Iter<Box<[u8]>>> {
        ParseArguments::new(self.arguments.iter())
    }
}

/// An iterator that parses arguments from bytes.
#[derive(Clone, Copy, Default, Debug, Hash)]
pub struct ParseArguments<I>(I);

impl<'a, I: Iterator<Item = &'a Bytes>, Bytes: AsRef<[u8]> + 'a> ParseArguments<I> {
    /// Creates a new instance of `ParseArguments`.
    pub fn new(iter: I) -> Self {
        Self(iter)
    }

    /// Retrieves the inner iterator.
    pub fn into_inner(self) -> I {
        self.0
    }
}

impl<'a, I: Iterator<Item = &'a Bytes>, Bytes: AsRef<[u8]> + 'a> Iterator for ParseArguments<I> {
    type Item = serde_json::Result<ValueOwned>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(AsRef::as_ref).map(serde_json::from_slice)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        fn parse<'a, B, Bytes: AsRef<[u8]>>(
            mut f: impl FnMut(B, serde_json::Result<ValueOwned>) -> B,
        ) -> impl FnMut(B, &'a Bytes) -> B {
            move |accum, bytes| f(accum, serde_json::from_slice(bytes.as_ref()))
        }
        self.0.fold(init, parse(f))
    }

    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        self.0
            .map(AsRef::as_ref)
            .map(serde_json::from_slice)
            .collect()
    }
}
