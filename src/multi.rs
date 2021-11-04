//! ## Multi-party signing
//!
//! This module contains various definitions that make multi-party signing easier.
//!
//! [`PartyBuilder`] makes it easy to build a transaction for signing with different rules.
//!
//! [`SigningParty`] is the simplest party. It computes a hash every time you want to sign it.
//!
//! [`PreHashedParty`] computes and stores the payload hash so it does not need to be recomputed.
//!
//! Both party types implement the common interface, the [`Party`] trait.

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use rlp::RlpStream;

use crate::access::{
    AccountResponse, BlockHeaderResponse, GetAccountAtLatestBlockRequest,
    GetLatestBlockHeaderRequest,
};
use crate::account::PADDED_TRANSACTION_DOMAIN_TAG;
use crate::algorithms::{FlowHasher, FlowSigner};
use crate::client::GrpcClient;
use crate::error::BoxError;
use crate::prelude::Account;
use crate::protobuf::Seal;
use crate::transaction::rlp::{rlp_encode_transaction_envelope, rlp_encode_transaction_payload};
use crate::transaction::{ProposalKeyE, SignatureE, TransactionE};

mod private {
    use crate::algorithms::FlowHasher;

    pub trait Sealed {}
    impl<H: FlowHasher> Sealed for super::PreHashedParty<H> {}
    impl Sealed for super::SigningParty {}
}

/// After envelope signatures are fed to a party, it turns into a transaction.
///
/// This is the exact type a party will turn in to.
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

/// The `Party` trait. You can get information about the transaction you are signing and sign it by
/// accepting some type that implements this trait.
///
/// This is `Sealed`, which means no foreign types may implement it, and it is not **Object safe**,
/// so no one can create bad behaving trait objects which can make this trait insecure.
pub trait Party<H: FlowHasher>: Sized + private::Sealed {
    /// Gets the script of the transaction.
    fn script(&self) -> &str;

    /// Gets the arguments of the transaction.
    fn arguments(&self) -> &[Box<[u8]>];

    /// Gets the reference block of the transaction.
    fn reference_block(&self) -> &[u8];

    /// Gets the gas limit of the transaction.
    fn gas_limit(&self) -> u64;

    /// Gets the address of the proposer of the transaction.
    fn proposer_address(&self) -> &[u8];

    /// Gets the key ID number of the proposal key.
    fn proposal_key_id(&self) -> u64;

    /// Gets the sequence number of the proposal key.
    fn proposer_sequence_number(&self) -> u64;

    /// Gets the address of the payer of the transaction.
    fn payer(&self) -> &[u8];

    /// Gets the addresses of the authorizers of the transaciton.
    fn authorizers(&self) -> &[Box<[u8]>];

    /// Computes the hash of the payload.
    fn payload(&self) -> H;

    /// Adds a payload signature.
    fn add_payload_signature(
        &mut self,
        signer_address: Box<[u8]>,
        signer_id: u32,
        signature: [u8; 64],
    );

    /// Computes the hash of the envelope.
    fn envelope(&self) -> H;

    /// Creates [`PartyTransaction`] by feeding this party with envelope signatures.
    fn into_transaction_with_envelope_signatures<SigAddr, Sig>(
        self,
        signatures: impl IntoIterator<Item = SignatureE<SigAddr, Sig>>,
    ) -> PartyTransaction<SigAddr, Sig>;
}

/// A builder that makes it easy to create new [`SigningParty`] instances.
///
/// ```
/// # use flow_sdk::multi::{PartyBuilder, SigningParty};
/// # use cadence_json::ValueRef;
///
/// let party = PartyBuilder::new()
///     .script("s")
///     .reference_block([0])
///     .gas_limit(123)
///     .proposer_address([1])
///     .proposal_key_id(2)
///     .proposal_key_sequence_number(3)
///     .payer([4])
///     .authorizer([5])
///     .build();
///
/// let party2 = SigningParty::new("s".into(), [].into(), [0].into(), 123, [1].into(), 2, 3, [4].into(), [[5].into()].into());
///
/// assert_eq!(party, party2);
/// ```
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
    /// Creates a new `PartyBuilder` with the default gas limit of `1000`.
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

    /// Sets the script of the transaction.
    pub fn script(mut self, script: impl Into<Box<str>>) -> Self {
        self.script = Some(script.into());
        self
    }

    /// Appends a new argument.
    pub fn argument(self, argument: impl serde::Serialize) -> Self {
        self.argument_raw(serde_json::to_vec(&argument).unwrap())
    }

    /// Appends arguments.
    pub fn arguments(self, arguments: impl IntoIterator<Item = impl serde::Serialize>) -> Self {
        self.arguments_raw(
            arguments
                .into_iter()
                .map(|val| serde_json::to_vec(&val).unwrap()),
        )
    }

    /// Appends a new UTF-8 encoded argument in Cadence JSON interchange format.
    pub fn argument_raw(mut self, argument: impl Into<Box<[u8]>>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    /// Appends raw UTF-8 encoded arguments in Cadence JSON interchange format.
    pub fn arguments_raw(
        mut self,
        arguments: impl IntoIterator<Item = impl Into<Box<[u8]>>>,
    ) -> Self {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Sets the reference block for this transaction.
    pub fn reference_block(mut self, reference_block: impl Into<Box<[u8]>>) -> Self {
        self.reference_block = Some(reference_block.into());
        self
    }

    /// Uses the latest block as the reference block for this transaction.
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

    /// Sets the gas limit of this transation.
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    /// Sets the address of the account that proposes this transaction.
    pub fn proposer_address(mut self, addr: impl Into<Box<[u8]>>) -> Self {
        self.proposer_address = Some(addr.into());
        self
    }

    /// Sets the key id of the proposal key of this transaction.
    pub fn proposal_key_id(mut self, id: u64) -> Self {
        self.proposal_key_id = Some(id);
        self
    }

    /// Sets the sequence number of the proposal key of this transaction.
    pub fn proposal_key_sequence_number(mut self, sequence_number: u64) -> Self {
        self.proposal_key_sequence_number = Some(sequence_number);
        self
    }

    /// Sets the address, key id and the sequence number by querying on the network about
    /// a logged-in account.
    pub async fn proposer_account<'a, C, Sk, Sign, Hash>(
        mut self,
        acc: &'a mut Account<C, Sk, Sign, Hash>,
    ) -> Result<Self, BoxError>
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

    /// Sets the address of the account that will pay for this transaction.
    pub fn payer(mut self, address: impl Into<Box<[u8]>>) -> Self {
        self.payer = Some(address.into());
        self
    }

    /// Records the address of the account that will pay for this transaction.
    #[inline]
    pub fn payer_account<C, S, Si, H>(self, acc: &Account<C, S, Si, H>) -> Self {
        self.payer(acc.address())
    }

    /// Appends the address of an account which authorizes this transaction.
    pub fn authorizer(mut self, address: impl Into<Box<[u8]>>) -> Self {
        self.authorizers.push(address.into());
        self
    }

    /// Appends the addresses of accounts that authorizes this transaction.
    pub fn authorizers(
        mut self,
        addresses: impl IntoIterator<Item = impl Into<Box<[u8]>>>,
    ) -> Self {
        self.authorizers
            .extend(addresses.into_iter().map(Into::into));
        self
    }

    /// Appends the address of an account which authorizers this transaction.
    #[inline]
    pub fn authorizer_account<C, S, Si, H>(self, acc: &Account<C, S, Si, H>) -> Self {
        self.authorizer(acc.address())
    }

    /// Appends the addresses of accounts that authorizes this transaction.
    #[inline]
    pub fn authorizer_accounts<'a, C: 'a, S: 'a, Si: 'a, H: 'a>(
        self,
        acc: impl IntoIterator<Item = &'a Account<C, S, Si, H>>,
    ) -> Self {
        self.authorizers(acc.into_iter().map(Account::address))
    }

    /// Builds a [`SigningParty`] from this builder, assuming all fields have been set.
    pub fn build(self) -> SigningParty {
        SigningParty::new(
            self.script.unwrap(),
            self.arguments.into(),
            self.reference_block.unwrap(),
            self.gas_limit,
            self.proposer_address.unwrap(),
            self.proposal_key_id.unwrap(),
            self.proposal_key_sequence_number.unwrap(),
            self.payer.unwrap(),
            self.authorizers.into(),
        )
    }

    /// Builds a [`PreHashedParty`] from this builder, assuming all fields have been set.
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    signer_map: HashMap<Box<[u8]>, u32>,
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

    fn proposal_key_id(&self) -> u64 {
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
            self.payload_signatures.iter().map(|sig| {
                (
                    *self.signer_map.get(&sig.address).unwrap(),
                    sig.key_id,
                    &sig.signature,
                )
            }),
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
    /// Builds a signer address to signer index map.
    pub fn build_signer_map(
        proposer: &[u8],
        payer: &[u8],
        authorizers: &[impl AsRef<[u8]>],
    ) -> HashMap<Box<[u8]>, u32> {
        let mut map: HashMap<Box<[u8]>, u32> = HashMap::new();
        map.insert(proposer.into(), 0);

        let mut add = |addr: &[u8]| {
            let len = map.len();
            if let Entry::Vacant(entry) = map.entry(addr.into()) {
                entry.insert(len as u32);
            }
        };

        add(payer);

        for authorizer in authorizers {
            add(authorizer.as_ref());
        }

        map
    }

    /// Creates a new [`SigningParty`] with all the data associated to a transaction.
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
        let signer_map = Self::build_signer_map(&proposer_address, &payer, &authorizers);
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
            signer_map,
        }
    }

    /// Computes the payload hash of this party, and turns this into a [`PreHashedParty`].
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
///
/// Note that this can be a bit less secure than using [`SigningParty`].
/// A malicious attacker may modify the payload using unsafe pointer operations.
#[derive(Clone, PartialEq, Eq)]
pub struct PreHashedParty<H> {
    party: SigningParty,
    payload: H,
}

impl<H: FlowHasher> PreHashedParty<H> {
    /// Creates a new [`PreHashedParty`] with all the data associated to a transaction.
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

    fn proposal_key_id(&self) -> u64 {
        <SigningParty as Party<H>>::proposal_key_id(&self.party)
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
