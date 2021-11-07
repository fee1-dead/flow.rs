//! Mutate the blockchain with your private key(s).
//!
//! This module contains ways to log in to an account via secret key(s) and an Access API client.

use std::collections::HashMap;
use std::iter::empty;
use std::marker::PhantomData;
use std::slice;

use crate::access::{
    AccountResponse, BlockHeaderResponse, GetAccountAtLatestBlockRequest,
    GetLatestBlockHeaderRequest, SendTransactionRequest, SendTransactionResponse,
};
use crate::algorithms::{
    DefaultHasher, DefaultSecretKey, DefaultSigner, FlowHasher, FlowSigner, HashAlgorithm,
    Signature, SignatureAlgorithm,
};
use crate::client::{FlowClient, GrpcClient};
use crate::entities::AccountKey;
use crate::error::BoxError;
use crate::multi::{Party, PartyTransaction};
use crate::protobuf::Seal;
use crate::sign::{KeyIdIter, MkSigIter, Multi, One, SignIter, SignMethod};
use crate::transaction::rlp::rlp_encode_transaction_envelope;
use crate::transaction::{ProposalKeyE, SignatureE, TransactionE, TransactionHeader};

const PADDED_LEN: usize = 32;

/// The transaction domain tag, padded to 32 bytes.
pub const PADDED_TRANSACTION_DOMAIN_TAG: [u8; PADDED_LEN] =
    padded::<PADDED_LEN>(b"FLOW-V0.0-transaction");

pub use crate::error::AccountError as Error;

/// An account that uses the default signing and hashing algorithms.
pub type DefaultAccount<Client> = Account<Client>;

#[derive(Clone)]
/// An account.
///
/// This is your gateway to making transactions, as this holds the secret keys necessary for signing them, as well
/// as the client, for sending any requests over the network.
pub struct Account<
    Client,
    SecretKey = DefaultSecretKey,
    Signer = DefaultSigner,
    Hasher = DefaultHasher,
> {
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

    /// Returns the signer.
    #[inline]
    pub fn signer(&self) -> &Sn {
        &self.signer
    }

    /// Returns the client.
    #[inline]
    pub fn client(&mut self) -> &mut FlowClient<Cl> {
        &mut self.client
    }

    /// Clones the client from this account.
    #[inline]
    pub fn client_cloned(&self) -> FlowClient<Cl>
    where
        Cl: Clone,
    {
        self.client.clone()
    }

    /// Returns the primary public key of this account.
    pub fn primary_public_key(&self) -> Sn::PublicKey
    where
        Sn: FlowSigner<SecretKey = Sk>,
    {
        self.signer
            .to_public_key(self.sign_method.primary_secret_key())
    }

    /// Returns the primary key id number of this account.
    #[inline]
    pub fn primary_key_id(&self) -> u32 {
        self.sign_method.primary_key_id()
    }
}

impl<Client, SecretKey, Signer, Hasher> Account<Client, SecretKey, Signer, Hasher>
where
    Signer: FlowSigner<SecretKey = SecretKey>,
    Hasher: FlowHasher,
{
    ///////////////////////
    // CONSTRUCTION

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
    pub async fn new<Addr>(
        client: Client,
        address: Addr,
        secret_key: SecretKey,
    ) -> Result<Self, Error>
    where
        Client: GrpcClient<GetAccountAtLatestBlockRequest<Addr>, AccountResponse>,
    {
        let mut client = FlowClient::new(client);
        let acc = client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;

        let crate::entities::Account { address, keys, .. } = acc;

        let mut account_key = None;

        let signer = Signer::new();
        let public_key = signer.to_public_key(&secret_key);
        let serialized = signer.serialize_public_key(&public_key);

        for key in keys {
            if *key.public_key == serialized {
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

    /// Logs in to the account with multiple keys, verifying that the keys and the address matches.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    ///  - the client returns any errors while making requests
    ///  - the secret keys does not add up to the full weight to be able to sign (weight < 1000)
    ///  - could not find any public key of the account that matches one of the the secret key supplied.
    ///  - there were duplicate secret keys supplied
    ///  - the algorithms for the signer and the hasher do not match with the public information of a key.
    pub async fn new_multisign<Addr>(
        client: Client,
        address: Addr,
        primary_index: usize,
        secret_keys: &[SecretKey],
    ) -> Result<Self, Error>
    where
        Client: GrpcClient<GetAccountAtLatestBlockRequest<Addr>, AccountResponse>,
        SecretKey: Clone,
    {
        assert!(
            secret_keys.len() > 1,
            "cannot have less than 2 secret keys specified for multisign"
        );

        let mut client = FlowClient::new(client);
        let acc = client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;

        let crate::entities::Account { address, keys, .. } = acc;

        assert!(
            primary_index < secret_keys.len(),
            "primary key must be valid"
        );

        let signer = Signer::new();
        let mut primary_key_idx = usize::MAX;
        let mut total_weight = 0;
        let mut found_keys = Vec::new();

        let mut add_key = |key_index: usize, key_id, weight| {
            if key_index == primary_index {
                primary_key_idx = found_keys.len();
            }

            found_keys.push(One {
                key_id,
                key: secret_keys[key_index].clone(),
            });

            total_weight += weight;
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

            for key in keys {
                if let Some(key_index) = public_keys_to_find.remove(&*key.public_key) {
                    add_key(key_index, key.index, key.weight);
                }
            }

            if !public_keys_to_find.is_empty() {
                return Err(Error::NoMatchingKeyFound);
            }
        } else {
            // Hashing can be expensive for small sets.
            let mut public_keys_to_find: Vec<_> = secret_keys
                .iter()
                .map(|sk| signer.to_public_key(sk))
                .map(|pk| signer.serialize_public_key(&pk))
                .collect();

            for key in keys {
                if let Some((index, _)) = public_keys_to_find
                    .iter()
                    .enumerate()
                    .find(|(_, pubkey)| *pubkey == &*key.public_key)
                {
                    // Do not allow duplicate secret keys
                    public_keys_to_find.swap_remove(index);
                    add_key(index, key.index, key.weight);
                }
            }

            if !public_keys_to_find.is_empty() {
                return Err(Error::NoMatchingKeyFound);
            }
        }

        if total_weight < 1000 {
            return Err(Error::NotEnoughWeight);
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

    /// Creates a new account without checking that the address has the specified key.
    ///
    /// # Safety
    ///
    /// This is unsafe because it can end up signing with a wrong/revoked key, which means
    /// transactions sent to the network could fail.
    ///
    /// Do not do this unless you are sure that the key(s) contained in the sign method matches
    /// the address.
    #[inline]
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

    ////////////////////
    // INFORMATION

    /// Queries the sequence number for the primary key from the network.
    pub async fn primary_key_sequence_number<'a>(&'a mut self) -> Result<u32, BoxError>
    where
        Client: GrpcClient<GetAccountAtLatestBlockRequest<&'a [u8]>, AccountResponse>,
    {
        let address = &*self.address;
        let public_key = self.signer.serialize_public_key(&self.primary_public_key());

        let acc = self
            .client
            .account_at_latest_block(address)
            .await
            .map_err(Into::into)?;
        for key in acc.keys {
            if *key.public_key == public_key {
                return Ok(key.sequence_number);
            }
        }
        unreachable!();
    }

    //////////////////
    // SIGNING

    /// Creates signature(s) using this account's public key(s), consuming a populated hasher.
    pub fn sign(&self, hasher: Hasher) -> SignIter<Signer> {
        Self::sign_(hasher, self.signer(), &self.sign_method)
    }

    /// Creates signature(s) using this account's public key(s), signing provided data.
    pub fn sign_data(&self, data: impl AsRef<[u8]>) -> SignIter<Signer> {
        let mut hasher = Hasher::new();
        hasher.update(&data);
        self.sign(hasher)
    }

    /// Signs a party, assuming that you have confirmed all the details of the party.
    pub fn sign_party<P: Party<Hasher>>(&self, party: &mut P)
    where
        Signer::Signature: Signature<Serialized = [u8; 64]>,
    {
        let signatures = self.sign(party.payload());
        let key_ids = self.sign_method.key_ids();
        for (sig, key_id) in signatures.zip(key_ids) {
            party.add_payload_signature(self.address.clone(), key_id, sig.serialize())
        }
    }

    /// Signs the party as the payer, thereby converting the party into a transaction, ready to be sent.
    pub fn sign_party_as_payer<P: Party<Hasher>>(
        &self,
        party: P,
    ) -> PartyTransaction<Box<[u8]>, [u8; 64]>
    where
        Signer::Signature: Signature<Serialized = [u8; 64]>,
    {
        assert_eq!(&*self.address, party.payer());
        let signatures = self.sign(party.envelope());
        let key_ids = self.sign_method.key_ids();

        party.into_transaction_with_envelope_signatures(signatures.zip(key_ids).map(
            |(sig, key_id)| SignatureE {
                address: self.address.clone(),
                key_id,
                signature: sig.serialize(),
            },
        ))
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
    ) -> Result<SendTransactionResponse, BoxError>
    where
        Client: for<'b> GrpcClient<GetAccountAtLatestBlockRequest<&'b [u8]>, AccountResponse>,
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
            .into_iter()
            .find(|key| *key.public_key == pub_key)
            .unwrap();
        let sequence_number = key.sequence_number as u64;

        let latest_block = self
            .client
            .latest_block_header(Seal::Sealed)
            .await
            .map_err(Into::into)?;

        let reference_block_id = &*latest_block.id;
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
            script: transaction.script.as_ref().as_ref(),
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

        Ok(self
            .client
            .send_transaction(transaction)
            .await
            .map_err(Into::into)?)
    }

    ///////////////
    /// PRIVATE

    fn sign_<'a>(
        hasher: Hasher,
        signer: &'a Signer,
        method: &'a SignMethod<SecretKey>,
    ) -> SignIter<'a, Signer> {
        SignIter::new(hasher.finalize(), signer, method)
    }

    #[allow(clippy::too_many_arguments)]
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
            empty::<(u32, u32, &[u8])>(),
        );

        let mut hasher = Hasher::new();
        hasher.update(&PADDED_TRANSACTION_DOMAIN_TAG);
        hasher.update(&s.out());

        Self::sign_(hasher, signer, method)
    }

    #[allow(clippy::too_many_arguments)]
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
            &header.script.as_ref(),
            &header.arguments,
            reference_block_id,
            sequence_number,
            gas_limit,
        )
    }
}

#[repr(transparent)]
#[doc(hidden)] // implementation details
pub struct SliceHelper<Item>([Item]);

impl<Item> SliceHelper<Item> {
    #[doc(hidden)]
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

#[cfg(test)]
pub(crate) fn test_pad(src: &[u8]) -> [u8; 32] {
    padded(src)
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
