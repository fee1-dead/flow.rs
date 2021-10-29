mod rlp;
use std::{marker::PhantomData, slice};

pub use self::rlp::*;

mod signing;
use cadence_json::ValueOwned;
pub use signing::*;

mod template;
pub use template::*;

mod finalize;
pub use finalize::*;

use otopr::encoding::EncodeAsRef;
use otopr::*;
use wire_types::*;

pub type RepSlice<'a, T> = Repeated<&'a [T]>;

#[derive(Enumeration, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Unknown = 0,
    Pending = 1,
    Finalized = 2,
    Executed = 3,
    Sealed = 4,
    Expired = 5,
}

#[derive(EncodableMessage, Clone, Copy, Debug, PartialEq, Eq)]
#[otopr(encode_where_clause(
    where
        Address: AsRef<[u8]>,
))]
pub struct ProposalKeyE<Address> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub address: Address,
    pub key_id: u32,
    pub sequence_number: u64,
}

#[derive(DecodableMessage, Default)]
pub struct ProposalKeyD {
    pub address: Vec<u8>,
    pub key_id: u32,
    pub sequence_number: u32,
}

#[derive(EncodableMessage, Clone, Copy, Debug, PartialEq, Eq)]
#[otopr(encode_where_clause(
    where
        Address: AsRef<[u8]>,
        Signature: AsRef<[u8]>,
))]
pub struct SignatureE<Address, Signature> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub address: Address,
    pub key_id: u32,
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub signature: Signature,
}

#[derive(DecodableMessage, Default)]
pub struct SignatureD {
    pub address: Box<[u8]>,
    pub key_id: u32,
    pub signature: Box<[u8]>,
}

#[derive(EncodableMessage, Clone, Debug, PartialEq, Eq)]
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
        for<'a> <&'a Arguments as IntoIterator>::IntoIter: Clone,
        Authorizers: HasItem,
        <Authorizers as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Authorizers: IntoIterator<Item = &'a <Authorizers as HasItem>::Item>,
        for<'a> <&'a Authorizers as IntoIterator>::IntoIter: Clone,
        PayloadSignatures: HasItem<Item = SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> &'a PayloadSignatures: IntoIterator<Item = &'a SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> <&'a PayloadSignatures as IntoIterator>::IntoIter: Clone,
        EnvelopeSignatures: HasItem<Item = SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> &'a EnvelopeSignatures: IntoIterator<Item = &'a SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> <&'a EnvelopeSignatures as IntoIterator>::IntoIter: Clone,
))]
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
    pub script: Script,
    #[otopr(encode_via(wire_types::LengthDelimitedWire, <&Repeated<&Arguments>>::from(&x).map(|it| it.map(EncodeAsRef::new))))]
    pub arguments: Arguments,
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub reference_block_id: ReferenceBlockId,
    pub gas_limit: u64,
    pub proposal_key: ProposalKeyE<ProposalKeyAddress>,
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub payer: Payer,
    #[otopr(encode_via(wire_types::LengthDelimitedWire, <&Repeated<&Authorizers>>::from(&x).map(|it| it.map(AsRef::as_ref))))]
    pub authorizers: Authorizers,
    #[otopr(encode_via(wire_types::LengthDelimitedWire, <&Repeated<&PayloadSignatures>>::from(&x)))]
    pub payload_signatures: PayloadSignatures,
    #[otopr(encode_via(wire_types::LengthDelimitedWire, <&Repeated<&EnvelopeSignatures>>::from(&x)))]
    pub envelope_signatures: EnvelopeSignatures,
}

#[derive(DecodableMessage, Default)]
pub struct TransactionD {
    pub script: Box<[u8]>,
    pub arguments: Repeated<Vec<Box<[u8]>>>,
    pub reference_block_id: Box<[u8]>,
    pub gas_limit: u64,
    pub proposal_key: ProposalKeyD,
    pub payer: Box<[u8]>,
    pub authorizers: Repeated<Vec<Box<[u8]>>>,
    pub payload_signatures: Repeated<Vec<SignatureD>>,
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

pub struct ParseArguments<'a, I>(I, PhantomData<&'a ()>);

impl<I: Iterator> ParseArguments<'_, I> {
    pub fn new(iter: I) -> Self {
        Self(iter, PhantomData)
    }

    pub fn into_inner(self) -> I {
        self.0
    }
}

impl<'a, I: Iterator<Item = &'a Bytes>, Bytes: AsRef<[u8]> + 'a> Iterator
    for ParseArguments<'a, I>
{
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
