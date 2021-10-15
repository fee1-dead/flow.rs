use std::iter::empty;
use std::{error::Error as StdError, marker::PhantomData};

use crate::{
    Account, AccountKey, AccountResponse, BlockHeaderResponse, GetAccountAtLatestBlockRequest,
    GetLatestBlockHeaderRequest, ProposalKeyE, RepSlice, SendTransactionRequest,
    SendTransactionResponse, SignatureE, TransactionE, TransactionHeader,
};

use crate::algorithms::{FlowHasher, FlowSigner, HashAlgorithm, Signature, SignatureAlgorithm};

use crate::client::{FlowClient, GrpcClient};

const PADDED_LEN: usize = 32;
const PADDED_TRANSACTION_DOMAIN_TAG: [u8; PADDED_LEN] =
    padded::<PADDED_LEN>(b"FLOW-V0.0-transaction");

const fn padded<const N: usize>(src: &[u8]) -> [u8; N] {
    let mut new_buf = [0; N];

    let mut i = 0;

    while i < src.len() {
        new_buf[i] = src[i];
        i += 1;
    }

    new_buf
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not find a matching key for the private key.")]
    NoMatchingKeyFound,
    #[error("The hashing and signing algorithms do not match.")]
    AlgoMismatch,
    #[error("The key does not have enough weight.")]
    NotEnoughWeight,
    #[error(transparent)]
    Custom(#[from] Box<dyn StdError + Send + Sync>),
}

/// A simple account, no multisign.
#[derive(Clone)]
pub struct SimpleAccount<SecretKey, Signer, Hasher, Client> {
    address: Vec<u8>,
    key_id: u32,
    sequence_number: u32,
    secret_key: SecretKey,
    signer: Signer,
    client: FlowClient<Client>,
    _pd: PhantomData<Hasher>,
}

impl<Sk, Sn, Hs, Cl> SimpleAccount<Sk, Sn, Hs, Cl> {
    pub fn public_key(&self) -> Sn::PublicKey
    where
        Sn: FlowSigner<SecretKey = Sk>,
    {
        self.signer.to_public_key(&self.secret_key)
    }
    #[inline]
    pub fn address(&self) -> &[u8] {
        &self.address
    }
    #[inline]
    pub fn key_id(&self) -> u32 {
        self.key_id
    }
    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }
    #[inline]
    pub fn signer(&self) -> &Sn {
        &self.signer
    }
    #[inline]
    pub fn client(&mut self) -> &mut FlowClient<Cl> {
        &mut self.client
    }
}

impl<SecretKey, Signer, Hasher, Client> SimpleAccount<SecretKey, Signer, Hasher, Client>
where
    Signer: FlowSigner<SecretKey = SecretKey>,
    Hasher: FlowHasher,
{
    /// Sign a transaction with this account being the proposer, the payer and the only authorizer.
    ///
    /// Returns an envelope signature.
    pub fn sign_transaction(
        &self,
        script: impl AsRef<[u8]>,
        arguments: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
        reference_block_id: impl AsRef<[u8]>,
        gas_limit: u64,
    ) -> Signer::Signature {
        let mut s = rlp::RlpStream::new();
        crate::rlp_encode_transaction_envelope(
            &mut s,
            script,
            arguments,
            reference_block_id,
            gas_limit,
            self.address(),
            self.key_id(),
            self.sequence_number(),
            self.address(),
            [self.address()],
            empty::<(u32, u32, &[u8])>(),
        );

        let mut hasher = Hasher::new();
        hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
        hasher.update(&s.out());

        self.sign(hasher)
    }

    /// Sign a transaction header with a block id and gas limit.
    pub fn sign_transaction_header<const ARGS: usize>(
        &mut self,
        header: &TransactionHeader<ARGS>,
        reference_block_id: impl AsRef<[u8]>,
        gas_limit: u64,
    ) -> Signer::Signature {
        self.sign_transaction(
            &header.script,
            &header.arguments,
            reference_block_id,
            gas_limit,
        )
    }

    /// Creates a signature using this account's public key.
    pub fn sign(&self, hasher: Hasher) -> Signer::Signature {
        self.signer.sign(hasher, &self.secret_key)
    }

    pub async fn send_transaction_header<const ARGS: usize>(
        &mut self,
        transaction: &TransactionHeader<ARGS>,
    ) -> Result<SendTransactionResponse, Box<dyn StdError + Send + Sync>>
    where
        Client: GrpcClient<GetLatestBlockHeaderRequest, BlockHeaderResponse>,
        for<'a> Client: GrpcClient<SendTransactionRequest<'a>, SendTransactionResponse>,
    {
        let latest_block = self
            .client()
            .latest_block_header(true)
            .await
            .map_err(|e| e.into())?;
        let reference_block_id = &latest_block.0.into_inner().id;
        let args: Vec<&[u8]> = transaction.arguments.iter().map(|v| v.as_slice()).collect();
        let gas_limit = 1000;
        let sig = self.sign_transaction_header(transaction, reference_block_id, gas_limit);
        let sig = sig.serialize();
        let authorizers = [&self.address[..]];
        let envelope_signatures = [SignatureE {
            address: &self.address,
            key_id: self.key_id(),
            signature: sig.as_ref(),
        }];
        let transaction = TransactionE {
            script: &transaction.script,
            arguments: RepSlice::new(&args),
            reference_block_id,
            gas_limit,
            proposal_key: ProposalKeyE {
                address: &self.address,
                key_id: self.key_id,
                sequence_number: self.sequence_number() as u64,
            },
            payer: &self.address,
            authorizers: RepSlice::new(&authorizers),
            payload_signatures: RepSlice::new(&[]),
            envelope_signatures: RepSlice::new(&envelope_signatures),
        };
        self.client
            .send_transaction(transaction)
            .await
            .map_err(|e| e.into())
    }
}

impl<SecretKey, Signer, Hasher, Client> SimpleAccount<SecretKey, Signer, Hasher, Client>
where
    Signer: FlowSigner<SecretKey = SecretKey>,
    Hasher: FlowHasher,
    for<'a> Client: GrpcClient<GetAccountAtLatestBlockRequest<'a>, AccountResponse>,
{
    pub async fn new(
        client: Client,
        address: &'_ [u8],
        secret_key: SecretKey,
    ) -> Result<Self, Error> {
        let mut client = FlowClient::new(client);
        let acc = client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;
        assert_eq!(acc.account.address, address);

        let Account { address, keys, .. } = acc.account;

        let mut account_key = None;

        let signer = Signer::new();
        let public_key = signer.to_public_key(&secret_key);
        let serialized = signer.serialize_public_key(&public_key);

        for key in keys.0 {
            if key.public_key == serialized {
                account_key = Some(key);
            }
        }

        let AccountKey {
            index: key_id,
            sign_algo,
            hash_algo,
            weight,
            sequence_number,
            ..
        } = account_key.ok_or(Error::NoMatchingKeyFound)?;

        if weight != 1000 {
            return Err(Error::NotEnoughWeight);
        }

        if Signer::Algorithm::CODE != sign_algo || Hasher::Algorithm::CODE != hash_algo {
            return Err(Error::AlgoMismatch);
        }

        Ok(Self {
            address,
            key_id,
            sequence_number,
            secret_key,
            signer,
            client,
            _pd: PhantomData,
        })
    }
}

#[cfg(all(feature = "secp256k1-sign", feature = "sha3-hash"))]
pub type SimpleSecp256k1Sha3Account<Client> = SimpleAccount<
    secp256k1::SecretKey,
    secp256k1::Secp256k1<secp256k1::SignOnly>,
    tiny_keccak::Sha3,
    Client,
>;
