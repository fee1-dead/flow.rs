use std::iter::empty;
use std::slice;
use std::{error::Error as StdError, marker::PhantomData};

use crate::{
    Account, AccountKey, AccountResponse, BlockHeaderResponse, GetAccountAtLatestBlockRequest,
    GetLatestBlockHeaderRequest, ProposalKeyE, SendTransactionRequest, SendTransactionResponse,
    SignatureE, TransactionE, TransactionHeader,
};

use crate::algorithms::{
    DefaultHasher, DefaultSigner, FlowHasher, FlowSigner, HashAlgorithm, Signature,
    SignatureAlgorithm,
};

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
    #[error("The key(s) does not have enough weight.")]
    NotEnoughWeight,
    #[error(transparent)]
    Custom(#[from] Box<dyn StdError + Send + Sync>),
}

/// A simple account, no multisign.
#[derive(Clone)]
pub struct SimpleAccount<Client, SecretKey, Signer = DefaultSigner, Hasher = DefaultHasher> {
    // The address of this account.
    address: Box<[u8]>,
    key_id: u32,
    secret_key: SecretKey,
    signer: Signer,
    client: FlowClient<Client>,
    _pd: PhantomData<Hasher>,
}

pub struct MultiSignAccount<Client, SecretKey, Signer = DefaultSigner, Hasher = DefaultHasher> {
    address: Box<[u8]>,
    primary_key_id: u32,
    primary_key_index: usize,
    secret_keys: Box<[SecretKey]>,
    signer: Signer,
    client: FlowClient<Client>,
    _pd: PhantomData<Hasher>
}

impl<Cl, Sk, Sn, Hs> SimpleAccount<Cl, Sk, Sn, Hs> {
    /// Returns the public key of this account.
    #[inline]
    pub fn public_key(&self) -> Sn::PublicKey
    where
        Sn: FlowSigner<SecretKey = Sk>,
    {
        self.signer.to_public_key(&self.secret_key)
    }

    /// Returns the key id number of this account.
    #[inline]
    pub fn key_id(&self) -> u32 {
        self.key_id
    }

    /// Returns the address of this account.
    #[inline]
    pub fn address(&self) -> &[u8] {
        &self.address
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

#[repr(transparent)]
#[doc(hidden)] // implementation details
pub struct SliceHelper<T, Item>(T, PhantomData<Item>);

impl<T: AsRef<[Item]>, Item> SliceHelper<T, Item> {
    pub fn new_ref(t: &T) -> &Self {
        unsafe { &*(t as *const T as *const Self) }
    }
}

impl<'a, 'b, T: AsRef<[Item]>, Item: 'a> IntoIterator for &'a &'b SliceHelper<T, Item> {
    type Item = &'a Item;

    type IntoIter = slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_ref().iter()
    }
}

impl<T: AsRef<[Item]>, Item> otopr::HasItem for &'_ SliceHelper<T, Item> {
    type Item = Item;
}

impl<Client, SecretKey, Signer, Hasher> SimpleAccount<Client, SecretKey, Signer, Hasher>
where
    Signer: FlowSigner<SecretKey = SecretKey>,
    Hasher: FlowHasher,
{
    /// Logs in to a simple account, verifying that the key and the address matches.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    ///  - the client returns any errors while making requests
    ///  - the secret key does not have the full weight to be able to act on its own (weight < 1000)
    ///  - could not find any public key of the account that matches the secret key supplied.
    ///  - the algorithms for the signer and the hasher do not match with the public information of the key.
    pub async fn new(
        client: Client,
        address: &'_ [u8],
        secret_key: SecretKey,
    ) -> Result<Self, Error>
    where for<'a> Client: GrpcClient<GetAccountAtLatestBlockRequest<'a>, AccountResponse>,
    {
        let mut client = FlowClient::new(client);
        let acc = client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;
        assert_eq!(&*acc.account.address, address);

        let Account { address, keys, .. } = acc.account;

        let mut account_key = None;

        let signer = Signer::new();
        let public_key = signer.to_public_key(&secret_key);
        let serialized = signer.serialize_public_key(&public_key);

        for key in keys.into_inner() {
            if key.public_key == serialized {
                account_key = Some(key);
            }
        }

        let AccountKey {
            index: key_id,
            sign_algo,
            hash_algo,
            weight,
            ..
        } = account_key.ok_or(Error::NoMatchingKeyFound)?;

        if weight < 1000 {
            return Err(Error::NotEnoughWeight);
        }

        if Signer::Algorithm::CODE != sign_algo || Hasher::Algorithm::CODE != hash_algo {
            return Err(Error::AlgoMismatch);
        }

        Ok(Self {
            address,
            key_id,
            secret_key,
            signer,
            client,
            _pd: PhantomData,
        })
    }

    /// Sign a transaction with this account being the proposer, the payer and the only authorizer.
    ///
    /// Returns an envelope signature.
    pub fn sign_transaction(
        &self,
        script: impl AsRef<[u8]>,
        arguments: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
        reference_block_id: impl AsRef<[u8]>,
        sequence_number: u64,
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
            self.key_id() as u64,
            sequence_number,
            self.address(),
            [self.address()],
            empty::<(&[u8], u32, &[u8])>(),
        );

        let mut hasher = Hasher::new();
        hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
        hasher.update(&s.out());

        self.sign(hasher)
    }

    /// Sign a transaction header with a block id and gas limit.
    pub fn sign_transaction_header<'a, Arguments>(
        &mut self,
        header: &'a TransactionHeader<Arguments>,
        reference_block_id: impl AsRef<[u8]>,
        sequence_number: u64,
        gas_limit: u64,
    ) -> Signer::Signature
    where
        &'a Arguments: IntoIterator,
        <&'a Arguments as IntoIterator>::IntoIter: ExactSizeIterator,
        <<&'a Arguments as IntoIterator>::IntoIter as Iterator>::Item: AsRef<[u8]>,
    {
        self.sign_transaction(
            &header.script,
            &header.arguments,
            reference_block_id,
            sequence_number,
            gas_limit,
        )
    }

    /// Creates a signature using this account's public key, consuming a populated hasher.
    pub fn sign(&self, hasher: Hasher) -> Signer::Signature {
        self.signer.sign(hasher, &self.secret_key)
    }

    /// Send a transaction to the network. Signs the transaction header with a gas limit of 1000
    /// and using the latest sealed block as a reference.
    /// 
    /// Note that this does not increment the sequence number.
    ///
    /// # Errors
    ///
    /// This function returns an error if the client returns any errors when making requests.
    pub async fn send_transaction_header<'a, Arguments>(
        &'a mut self,
        transaction: &'a TransactionHeader<Arguments>,
    ) -> Result<SendTransactionResponse, Box<dyn StdError + Send + Sync>>
    where
        Client: for<'b> GrpcClient<GetAccountAtLatestBlockRequest<'b>, AccountResponse>,
        Client: GrpcClient<GetLatestBlockHeaderRequest, BlockHeaderResponse>,
        for<'b> Client: GrpcClient<
            SendTransactionRequest<
                &'a [u8],
                &'a SliceHelper<Arguments, Vec<u8>>,
                &'b [u8],
                &'a [u8],
                &'a [u8],
                [&'a [u8]; 1],
                [SignatureE<&'a [u8], &'a [u8]>; 0],
                [SignatureE<&'a [u8], &'b [u8]>; 1],
            >,
            SendTransactionResponse,
        >,
        Arguments: AsRef<[Vec<u8>]>,
        &'a Arguments: IntoIterator,
        <&'a Arguments as IntoIterator>::IntoIter: ExactSizeIterator,
        <<&'a Arguments as IntoIterator>::IntoIter as Iterator>::Item: AsRef<[u8]>,
    {
        let address = &self.address;
        let acc = self.client.account_at_latest_block(address).await.map_err(Into::into)?.account;
        let pub_key = self.signer().serialize_public_key(&self.public_key());
        let key = acc.keys.into_inner().into_iter().find(|key| key.public_key == pub_key ).unwrap();
        let sequence_number = key.sequence_number as u64;

        let latest_block = self
            .client
            .latest_block_header(true)
            .await
            .map_err(|e| e.into())?;

        let latest_block = latest_block.0;
        let reference_block_id = latest_block.id.as_slice();
        let gas_limit = 1000;
        let sig = self.sign_transaction_header(transaction, reference_block_id, sequence_number, gas_limit);
        let sig = sig.serialize();
        let envelope_signatures = [SignatureE {
            address: &self.address[..],
            key_id: self.key_id(),
            signature: sig.as_ref(),
        }];
        let transaction = TransactionE {
            script: transaction.script.as_ref(),
            arguments: SliceHelper::new_ref(&transaction.arguments),
            reference_block_id,
            gas_limit,
            proposal_key: ProposalKeyE {
                address: &self.address[..],
                key_id: self.key_id,
                sequence_number: sequence_number,
            },
            payer: &self.address[..],
            authorizers: [&self.address[..]],
            payload_signatures: [],
            envelope_signatures,
        };
        self.client
            .send_transaction(transaction)
            .await
            .map_err(|e| e.into())
    }
}
