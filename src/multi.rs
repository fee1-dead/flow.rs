use std::error::Error;

use cadence_json::ValueRef;
use rlp::RlpStream;

use crate::access::AccountResponse;
use crate::access::BlockHeaderResponse;
use crate::access::GetAccountAtLatestBlockRequest;
use crate::access::GetLatestBlockHeaderRequest;
use crate::account::PADDED_TRANSACTION_DOMAIN_TAG;
use crate::algorithms::FlowHasher;
use crate::algorithms::FlowSigner;
use crate::client::GrpcClient;
use crate::prelude::Account;
use crate::protobuf::Seal;
use crate::transaction::rlp_encode_transaction_envelope;
use crate::transaction::rlp_encode_transaction_payload;
use crate::transaction::ProposalKeyE;
use crate::transaction::SignatureE;
use crate::transaction::TransactionE;

mod private {
    use crate::algorithms::FlowHasher;

    pub trait Sealed {}
    impl<H: FlowHasher> Sealed for super::PreHashedParty<H> {}
    impl Sealed for super::SigningParty {}
}

pub type PartyTransaction<SigAddr, Sig> = TransactionE<
    Box<[u8]>,
    Vec<Box<[u8]>>,
    Box<[u8]>,
    Box<[u8]>,
    Box<[u8]>,
    Vec<Box<[u8]>>,
    Vec<SignatureE<Box<[u8]>, [u8; 64]>>,
    Vec<SignatureE<SigAddr, Sig>>,
>;

pub trait Party<H: FlowHasher>: Sized + private::Sealed {
    fn script(&self) -> &str;

    fn arguments(&self) -> &[Box<[u8]>];

    fn reference_block(&self) -> &[u8];

    fn gas_limit(&self) -> u64;

    fn proposer_address(&self) -> &[u8];

    fn proposer_key_id(&self) -> u64;

    fn proposer_sequence_number(&self) -> u64;

    fn payer(&self) -> &[u8];

    fn authorizers(&self) -> &[Box<[u8]>];

    fn payload(&self) -> H;

    fn add_payload_signature(
        &mut self,
        signer_address: Box<[u8]>,
        signer_id: u32,
        signature: [u8; 64],
    );

    fn envelope(&self) -> H;

    fn into_transaction_with_envelope_signatures<SigAddr, Sig>(
        self,
        signatures: impl IntoIterator<Item = SignatureE<SigAddr, Sig>>,
    ) -> PartyTransaction<SigAddr, Sig>;
}

#[derive(Clone)]
pub struct PartyBuilder {
    script: Option<Box<str>>,
    arguments: Vec<Box<[u8]>>,
    reference_block: Option<Box<[u8]>>,
    gas_limit: u64,
    proposer_address: Option<Box<[u8]>>,
    proposal_key_id: Option<u64>,
    proposal_key_sequence_number: Option<u64>,
    payer: Option<Box<[u8]>>,
    authorizers: Vec<Box<[u8]>>,
}

impl PartyBuilder {
    #[inline]
    pub const fn new() -> Self {
        Self {
            script: None,
            arguments: Vec::new(),
            reference_block: None,
            gas_limit: 1000,
            proposer_address: None,
            proposal_key_id: None,
            proposal_key_sequence_number: None,
            payer: None,
            authorizers: Vec::new(),
        }
    }

    pub fn script(mut self, script: impl Into<Box<str>>) -> Self {
        self.script = Some(script.into());
        self
    }

    pub fn argument<'a>(self, argument: impl AsRef<ValueRef<'a>>) -> Self {
        self.argument_raw(serde_json::to_vec(argument.as_ref()).unwrap())
    }

    pub fn arguments<'a>(
        self,
        arguments: impl IntoIterator<Item = impl AsRef<ValueRef<'a>>>,
    ) -> Self {
        self.arguments_raw(
            arguments
                .into_iter()
                .map(|val| serde_json::to_vec(val.as_ref()).unwrap()),
        )
    }

    pub fn argument_raw(mut self, argument: impl Into<Box<[u8]>>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn arguments_raw(
        mut self,
        arguments: impl IntoIterator<Item = impl Into<Box<[u8]>>>,
    ) -> Self {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    pub fn reference_block(mut self, reference_block: impl Into<Box<[u8]>>) -> Self {
        self.reference_block = Some(reference_block.into());
        self
    }

    pub async fn latest_block_as_reference<
        C: GrpcClient<GetLatestBlockHeaderRequest, BlockHeaderResponse>,
    >(
        mut self,
        client: &mut C,
    ) -> Result<Self, C::Error> {
        self.reference_block = Some(
            client
                .send(GetLatestBlockHeaderRequest { seal: Seal::Sealed })
                .await?
                .0
                .id,
        );
        Ok(self)
    }

    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    pub fn proposer_address(mut self, addr: impl Into<Box<[u8]>>) -> Self {
        self.proposer_address = Some(addr.into());
        self
    }

    pub fn proposal_key_id(mut self, id: u64) -> Self {
        self.proposal_key_id = Some(id);
        self
    }

    pub fn proposal_key_sequence_number(mut self, sequence_number: u64) -> Self {
        self.proposal_key_sequence_number = Some(sequence_number);
        self
    }

    pub async fn proposer_account<'a, C, Sk, Sign, Hash>(
        mut self,
        acc: &'a mut Account<C, Sk, Sign, Hash>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>>
    where
        C: GrpcClient<GetAccountAtLatestBlockRequest<'a>, AccountResponse>,
        Sign: FlowSigner<SecretKey = Sk>,
        Hash: FlowHasher,
    {
        self.proposer_address = Some(acc.address().into());
        self.proposal_key_id = Some(acc.primary_key_id().into());
        self.proposal_key_sequence_number = Some(acc.primary_key_sequence_number().await?.into());
        Ok(self)
    }

    pub fn payer(mut self, address: impl Into<Box<[u8]>>) -> Self {
        self.payer = Some(address.into());
        self
    }

    #[inline]
    pub fn payer_account<C, S, Si, H>(self, acc: &Account<C, S, Si, H>) -> Self {
        self.payer(acc.address())
    }

    pub fn authorizer(mut self, address: impl Into<Box<[u8]>>) -> Self {
        self.authorizers.push(address.into());
        self
    }

    pub fn authorizers(
        mut self,
        addresses: impl IntoIterator<Item = impl Into<Box<[u8]>>>,
    ) -> Self {
        self.authorizers
            .extend(addresses.into_iter().map(Into::into));
        self
    }

    #[inline]
    pub fn authorizer_account<C, S, Si, H>(self, acc: &Account<C, S, Si, H>) -> Self {
        self.authorizer(acc.address())
    }

    #[inline]
    pub fn authorizer_accounts<'a, C: 'a, S: 'a, Si: 'a, H: 'a>(
        self,
        acc: impl IntoIterator<Item = &'a Account<C, S, Si, H>>,
    ) -> Self {
        self.authorizers(acc.into_iter().map(Account::address))
    }

    pub fn build(self) -> SigningParty {
        SigningParty {
            script: self.script.unwrap(),
            arguments: self.arguments.into(),
            reference_block: self.reference_block.unwrap(),
            gas_limit: self.gas_limit,
            proposer_address: self.proposer_address.unwrap(),
            proposal_key_id: self.proposal_key_id.unwrap(),
            proposal_key_sequence_number: self.proposal_key_sequence_number.unwrap(),
            payer: self.payer.unwrap(),
            authorizers: self.authorizers.into(),
            payload_signatures: Vec::new(),
        }
    }

    #[inline]
    pub fn build_prehashed<H: FlowHasher>(self) -> PreHashedParty<H> {
        self.build().into_prehashed()
    }
}

impl Default for PartyBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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
    payload_signatures: Vec<SignatureE<Box<[u8]>, [u8; 64]>>,
}

impl SigningParty {
    pub fn get_address(&self, index: usize) -> &[u8] {
        match index {
            0 => &self.proposer_address,
            1 => &self.payer,
            index => &self.authorizers[index - 2],
        }
    }
}

impl<H: FlowHasher> Party<H> for SigningParty {
    fn script(&self) -> &str {
        &self.script
    }

    fn arguments(&self) -> &[Box<[u8]>] {
        &self.arguments
    }

    fn reference_block(&self) -> &[u8] {
        &self.reference_block
    }

    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    fn proposer_address(&self) -> &[u8] {
        &self.proposer_address
    }

    fn proposer_key_id(&self) -> u64 {
        self.proposal_key_id
    }

    fn proposer_sequence_number(&self) -> u64 {
        self.proposal_key_sequence_number
    }

    fn payer(&self) -> &[u8] {
        &self.payer
    }

    fn authorizers(&self) -> &[Box<[u8]>] {
        &self.authorizers
    }

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

    fn add_payload_signature(
        &mut self,
        signer_address: Box<[u8]>,
        signer_id: u32,
        signature: [u8; 64],
    ) {
        self.payload_signatures.push(SignatureE {
            address: signer_address,
            key_id: signer_id,
            signature,
        });
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
                .map(|sig| (&sig.address, sig.key_id, &sig.signature)),
        );
        hasher.update(&stream.out());
        hasher
    }

    fn into_transaction_with_envelope_signatures<SigAddr, Sig>(
        self,
        signatures: impl IntoIterator<Item = SignatureE<SigAddr, Sig>>,
    ) -> PartyTransaction<SigAddr, Sig> {
        TransactionE {
            script: self.script.into_boxed_bytes(),
            arguments: self.arguments.into(),
            reference_block_id: self.reference_block,
            gas_limit: self.gas_limit,
            proposal_key: ProposalKeyE {
                address: self.proposer_address,
                key_id: self.proposal_key_id as u32,
                sequence_number: self.proposal_key_sequence_number,
            },
            payer: self.payer,
            authorizers: self.authorizers.into(),
            payload_signatures: self.payload_signatures,
            envelope_signatures: signatures.into_iter().collect(),
        }
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

    pub fn into_prehashed<H: FlowHasher>(self) -> PreHashedParty<H> {
        let payload = self.payload();
        PreHashedParty {
            party: self,
            payload,
        }
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

    fn add_payload_signature(
        &mut self,
        signer_address: Box<[u8]>,
        signer_id: u32,
        signature: [u8; 64],
    ) {
        <SigningParty as Party<H>>::add_payload_signature(
            &mut self.party,
            signer_address,
            signer_id,
            signature,
        )
    }

    fn envelope(&self) -> H {
        self.party.envelope()
    }

    fn script(&self) -> &str {
        <SigningParty as Party<H>>::script(&self.party)
    }

    fn arguments(&self) -> &[Box<[u8]>] {
        <SigningParty as Party<H>>::arguments(&self.party)
    }

    fn reference_block(&self) -> &[u8] {
        <SigningParty as Party<H>>::reference_block(&self.party)
    }

    fn gas_limit(&self) -> u64 {
        <SigningParty as Party<H>>::gas_limit(&self.party)
    }

    fn proposer_address(&self) -> &[u8] {
        <SigningParty as Party<H>>::proposer_address(&self.party)
    }

    fn proposer_key_id(&self) -> u64 {
        <SigningParty as Party<H>>::proposer_key_id(&self.party)
    }

    fn proposer_sequence_number(&self) -> u64 {
        <SigningParty as Party<H>>::proposer_sequence_number(&self.party)
    }

    fn payer(&self) -> &[u8] {
        <SigningParty as Party<H>>::payer(&self.party)
    }

    fn authorizers(&self) -> &[Box<[u8]>] {
        <SigningParty as Party<H>>::authorizers(&self.party)
    }

    fn into_transaction_with_envelope_signatures<SigAddr, Sig>(
        self,
        signatures: impl IntoIterator<Item = SignatureE<SigAddr, Sig>>,
    ) -> PartyTransaction<SigAddr, Sig> {
        <SigningParty as Party<H>>::into_transaction_with_envelope_signatures(
            self.party, signatures,
        )
    }
}
