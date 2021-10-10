use std::{error::Error as StdError, marker::PhantomData};

use crate::{Account, AccountKey, AccountResponse, GetAccountAtLatestBlockRequest};

use crate::algorithms::{FlowHasher, FlowSigner, HashAlgorithm, SignatureAlgorithm};

use crate::client::{FlowClient, GrpcClient};

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
    #[inline]
    pub fn address(&self) -> &[u8] {
        &self.address
    }
    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
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

        let Account {
            address,
            keys,
            ..
        } = acc.account;

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
            return Err(Error::AlgoMismatch)
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
