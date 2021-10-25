use std::collections::HashMap;
use std::iter::empty;
use std::slice;
use std::{error::Error as StdError, marker::PhantomData};

use crate::protobuf::Seal;
use crate::sign::{KeyIdIter, MkSigIter, Multi, One, SignIter, SignMethod};

use crate::access::{BlockHeaderResponse, GetLatestBlockHeaderRequest, SendTransactionRequest, GetAccountAtLatestBlockRequest, AccountResponse, SendTransactionResponse};
use crate::entities::AccountKey;
use crate::transaction::{TransactionHeader, ProposalKeyE, rlp_encode_transaction_envelope, TransactionE, SignatureE};

use crate::algorithms::{
    DefaultHasher, DefaultSigner, FlowHasher, FlowSigner, HashAlgorithm, Signature,
    SignatureAlgorithm,
};

use crate::client::{FlowClient, GrpcClient};

const PADDED_LEN: usize = 32;
pub const PADDED_TRANSACTION_DOMAIN_TAG: [u8; PADDED_LEN] =
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
#[non_exhaustive]
pub enum Error {
    #[error("Could not find a matching key for the private key.")]
    NoMatchingKeyFound,
    #[error("The hashing and signing algorithms do not match.")]
    AlgoMismatch,
    #[error("The key(s) does not have enough weight.")]
    NotEnoughWeight,
    #[error("A key was revoked.")]
    KeyRevoked,
    #[error(transparent)]
    Custom(#[from] Box<dyn StdError + Send + Sync>),
}

#[derive(Clone)]
pub struct Account<Client, SecretKey, Signer = DefaultSigner, Hasher = DefaultHasher> {
    // The address of this account.
    address: Box<[u8]>,
    sign_method: SignMethod<SecretKey>,
    signer: Signer,
    client: FlowClient<Client>,
    _pd: PhantomData<Hasher>,
}

impl<Cl, Sk, Sn, Hs> Account<Cl, Sk, Sn, Hs> {
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

    pub fn primary_public_key(&self) -> Sn::PublicKey
    where
        Sn: FlowSigner<SecretKey = Sk>,
    {
        self.signer
            .to_public_key(self.sign_method.primary_secret_key())
    }

    pub fn primary_key_id(&self) -> u32 {
        self.sign_method.primary_key_id()
    }
}

impl<Client, SecretKey, Signer, Hasher> Account<Client, SecretKey, Signer, Hasher>
where
    Signer: FlowSigner<SecretKey = SecretKey>,
    Hasher: FlowHasher,
{
    fn sign_<'a>(
        hasher: Hasher,
        signer: &'a Signer,
        method: &'a SignMethod<SecretKey>,
    ) -> SignIter<'a, Signer> {
        SignIter::new(hasher.finalize(), signer, method)
    }

    /// Creates a signature using this account's public key(s), consuming a populated hasher.
    pub fn sign(&self, hasher: Hasher) -> SignIter<Signer> {
        Self::sign_(hasher, self.signer(), &self.sign_method)
    }

    fn sign_transaction_<'a>(
        key_id: u32,
        address: &[u8],
        signer: &'a Signer,
        method: &'a SignMethod<SecretKey>,
        script: impl AsRef<[u8]>,
        arguments: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
        reference_block_id: impl AsRef<[u8]>,
        sequence_number: u64,
        gas_limit: u64,
    ) -> SignIter<'a, Signer> {
        let mut s = rlp::RlpStream::new();
        rlp_encode_transaction_envelope(
            &mut s,
            script,
            arguments,
            reference_block_id,
            gas_limit,
            address,
            key_id as u64,
            sequence_number,
            address,
            [address],
            empty::<(&[u8], u32, &[u8])>(),
        );

        let mut hasher = Hasher::new();
        hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
        hasher.update(&s.out());

        Self::sign_(hasher, signer, method)
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
    ) -> SignIter<Signer> {
        Self::sign_transaction_(
            self.primary_key_id(),
            &self.address,
            &self.signer,
            &self.sign_method,
            script,
            arguments,
            reference_block_id,
            sequence_number,
            gas_limit,
        )
    }

    fn sign_transaction_header_<'a, 'b, Arguments>(
        key_id: u32,
        address: &[u8],
        signer: &'a Signer,
        method: &'a SignMethod<SecretKey>,
        header: &'b TransactionHeader<Arguments>,
        reference_block_id: impl AsRef<[u8]>,
        sequence_number: u64,
        gas_limit: u64,
    ) -> SignIter<'a, Signer>
    where
        &'b Arguments: IntoIterator,
        <&'b Arguments as IntoIterator>::IntoIter: ExactSizeIterator,
        <<&'b Arguments as IntoIterator>::IntoIter as Iterator>::Item: AsRef<[u8]>,
    {
        Self::sign_transaction_(
            key_id,
            address,
            signer,
            method,
            &header.script,
            &header.arguments,
            reference_block_id,
            sequence_number,
            gas_limit,
        )
    }

    /// Sign a transaction header with a block id and gas limit.
    pub fn sign_transaction_header<'a, Arguments>(
        &self,
        header: &'a TransactionHeader<Arguments>,
        reference_block_id: impl AsRef<[u8]>,
        sequence_number: u64,
        gas_limit: u64,
    ) -> SignIter<Signer>
    where
        &'a Arguments: IntoIterator,
        <&'a Arguments as IntoIterator>::IntoIter: ExactSizeIterator,
        <<&'a Arguments as IntoIterator>::IntoIter as Iterator>::Item: AsRef<[u8]>,
    {
        Self::sign_transaction_header_(
            self.primary_key_id(),
            &self.address,
            &self.signer,
            &self.sign_method,
            header,
            reference_block_id,
            sequence_number,
            gas_limit,
        )
    }

    /// Logs in to the account with one key, verifying that the key and the address matches.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    ///  - the client returns any errors while making requests
    ///  - the secret key does not have the full weight to be able to act on its own (weight < 1000)
    ///  - could not find any public key of the account that matches the secret key supplied.
    ///  - the algorithms for the signer and the hasher do not match with the public information of the key.
    pub async fn new<'a>(
        client: Client,
        address: &'a [u8],
        secret_key: SecretKey,
    ) -> Result<Self, Error>
    where
        Client: GrpcClient<GetAccountAtLatestBlockRequest<'a>, AccountResponse>,
    {
        let mut client = FlowClient::new(client);
        let acc = client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;

        assert_eq!(&*acc.address, address);

        let crate::entities::Account { address, keys, .. } = acc;

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
            revoked,
            ..
        } = account_key.ok_or(Error::NoMatchingKeyFound)?;

        if revoked {
            return Err(Error::KeyRevoked);
        }

        if weight < 1000 {
            return Err(Error::NotEnoughWeight);
        }

        if Signer::Algorithm::CODE != sign_algo || Hasher::Algorithm::CODE != hash_algo {
            return Err(Error::AlgoMismatch);
        }

        Ok(Self {
            address,
            sign_method: SignMethod::One(One {
                key_id,
                key: secret_key,
            }),
            signer,
            client,
            _pd: PhantomData,
        })
    }

    pub async fn new_multisign<'a, Keys>(
        client: Client,
        address: &'a [u8],
        primary_index: usize,
        secret_keys: &[SecretKey],
    ) -> Result<Self, Error>
    where
        Client: GrpcClient<GetAccountAtLatestBlockRequest<'a>, AccountResponse>,
        SecretKey: Clone,
    {
        assert!(secret_keys.len() > 1, "cannot have less than 2 secret keys specified for multisign");

        let mut client = FlowClient::new(client);
        let acc = client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;

        assert_eq!(&*acc.address, address);

        let crate::entities::Account { address, keys, .. } = acc;

        assert!(
            primary_index < secret_keys.len(),
            "primary key must be valid"
        );

        let signer = Signer::new();
        let mut primary_key_idx = usize::MAX;
        let mut found_keys = Vec::new();

        let mut add_key = |key_index: usize, key_id| {
            if key_index == primary_index {
                primary_key_idx = found_keys.len();
            }

            found_keys.push(One {
                key_id,
                key: secret_keys[key_index].clone(),
            });
        };

        if secret_keys.len() > 10 {
            // Hash the large set of secret keys.
            let mut public_keys_to_find: HashMap<_, _> = secret_keys
                .iter()
                .enumerate()
                .map(|(idx, secret_key)| {
                    (
                        signer.serialize_public_key(&signer.to_public_key(secret_key)),
                        idx,
                    )
                })
                .collect();
                
            for key in keys.into_inner() {
                if let Some(key_index) = public_keys_to_find.remove(&*key.public_key) {
                    add_key(key_index, key.index);
                }
            }

            if !public_keys_to_find.is_empty() {
                return Err(Error::NoMatchingKeyFound);
            }
        } else {
            // Hashing can be expensive for small sets.
            let mut keys_found = 0;
            let public_keys_to_find: Vec<_> = secret_keys
                .iter()
                .map(|sk| signer.to_public_key(sk))
                .map(|pk| signer.serialize_public_key(&pk))
                .collect();

            for key in keys.into_inner() {
                if let Some((index, _)) = public_keys_to_find.iter().enumerate().find(|(_, pubkey)| *pubkey == &*key.public_key) {
                    keys_found += 1;
                    add_key(index, key.index);
                }
            }

            if keys_found != public_keys_to_find.len() {
                return Err(Error::NoMatchingKeyFound);
            }
        }

        Ok(Self {
            address,
            sign_method: SignMethod::Multi(Multi {
                primary_key_idx,
                keys: found_keys.into_boxed_slice(),
            }),
            signer,
            client,
            _pd: PhantomData,
        })
    }

    pub unsafe fn new_unchecked(
        client: Client,
        address: Box<[u8]>,
        sign_method: SignMethod<SecretKey>,
    ) -> Self {
        Self {
            address,
            sign_method,
            signer: Signer::new(),
            client: FlowClient::new(client),
            _pd: PhantomData,
        }
    }

    /// Send a transaction to the network. Signs the transaction header with a gas limit of 1000
    /// and using the latest sealed block as a reference.
    ///
    /// Note that this does not increment the sequence number.
    ///
    /// # Errors
    ///
    /// This function returns an error if the client returns any errors when making requests.
    pub async fn send_transaction_header<'a, Arguments, Argument>(
        &'a mut self,
        transaction: &'a TransactionHeader<Arguments>,
    ) -> Result<SendTransactionResponse, Box<dyn StdError + Send + Sync>>
    where
        Client: for<'b> GrpcClient<GetAccountAtLatestBlockRequest<'b>, AccountResponse>,
        Client: GrpcClient<GetLatestBlockHeaderRequest, BlockHeaderResponse>,
        for<'b> Client: GrpcClient<
            SendTransactionRequest<
                &'b [u8],
                &'b SliceHelper<Argument>,
                &'b [u8],
                &'b [u8],
                &'b [u8],
                [&'b [u8]; 1],
                [SignatureE<&'b [u8], &'b [u8]>; 0],
                EmitRefAndDropOnNext<
                    SignatureE<&'b [u8], <Signer::Signature as Signature>::Serialized>,
                    MkSigIter<'b, KeyIdIter<'b, Signer::SecretKey>, SignIter<'b, Signer>>,
                >,
            >,
            SendTransactionResponse,
        >,
        Arguments: AsRef<[Argument]>,
        Argument: AsRef<[u8]>,
        &'a Arguments: IntoIterator,
        <&'a Arguments as IntoIterator>::IntoIter: ExactSizeIterator,
        <<&'a Arguments as IntoIterator>::IntoIter as Iterator>::Item: AsRef<[u8]>,
    {
        let address = &*self.address;
        let acc = self
            .client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;
        let pub_key = self.primary_public_key();
        let pub_key = self.signer.serialize_public_key(&pub_key);
        let key = acc
            .keys
            .into_inner()
            .into_iter()
            .find(|key| key.public_key == pub_key)
            .unwrap();
        let sequence_number = key.sequence_number as u64;

        let latest_block = self
            .client
            .latest_block_header(Seal::Sealed)
            .await
            .map_err(|e| e.into())?;

        let reference_block_id = latest_block.id.as_slice();
        let gas_limit = 1000;
        let sig = Self::sign_transaction_header_(
            self.primary_key_id(),
            &self.address,
            &self.signer,
            &self.sign_method,
            transaction,
            reference_block_id,
            sequence_number,
            gas_limit,
        );

        let envelope_signatures = EmitRefAndDropOnNext(
            MkSigIter::new(&self.address, self.sign_method.key_ids(), sig),
            PhantomData,
        );
        let transaction = TransactionE {
            script: transaction.script.as_ref(),
            arguments: SliceHelper::new_ref(transaction.arguments.as_ref()),
            reference_block_id,
            gas_limit,
            proposal_key: ProposalKeyE {
                address: &*self.address,
                key_id: self.primary_key_id(),
                sequence_number,
            },
            payer: &*self.address,
            authorizers: [&*self.address],
            payload_signatures: [],
            envelope_signatures,
        };
        self.client
            .send_transaction(transaction)
            .await
            .map_err(|e| e.into())
    }
}

#[repr(transparent)]
#[doc(hidden)] // implementation details
pub struct SliceHelper<Item>([Item]);

impl<Item> SliceHelper<Item> {
    pub fn new_ref(t: &[Item]) -> &Self {
        unsafe { &*(t as *const [Item] as *const Self) }
    }
}

impl<'a, 'b, Item: 'a> IntoIterator for &'a &'b SliceHelper<Item> {
    type Item = &'a Item;

    type IntoIter = slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_ref().iter()
    }
}

impl<Item> otopr::HasItem for &'_ SliceHelper<Item> {
    type Item = Item;
}

#[doc(hidden)] // implementation details
#[derive(Clone)]
pub struct EmitRefAndDropOnNextIter<'a, T, I>(Option<T>, I, PhantomData<&'a T>);

#[doc(hidden)]
#[derive(Clone)]
pub struct EmitRefAndDropOnNext<T, I>(I, PhantomData<T>);

impl<'a, T, I: Iterator<Item = T> + Clone> IntoIterator for &'a EmitRefAndDropOnNext<T, I> {
    type Item = &'a T;

    type IntoIter = EmitRefAndDropOnNextIter<'a, T, I>;

    fn into_iter(self) -> Self::IntoIter {
        EmitRefAndDropOnNextIter(None, self.0.clone(), PhantomData)
    }
}

impl<'a, T, I: Iterator<Item = T>> Iterator for EmitRefAndDropOnNextIter<'a, T, I> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(it) = self.1.next() {
            let r = self.0.insert(it);

            // SAFETY: Must ensure that each item is dropped before calling `next()`.
            //
            // FIXME: Can we avoid this and work with the trait system instead?
            Some(unsafe { &*(r as *const T) })
        } else {
            None
        }
    }
}

impl<T, I> otopr::HasItem for EmitRefAndDropOnNext<T, I> {
    type Item = T;
}
