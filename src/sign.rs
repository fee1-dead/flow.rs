//! Common logic for signing transactions.

use std::iter::{FusedIterator, Map, Zip};
use std::slice;

use crate::algorithms::{FlowSigner, Signature};
use crate::transaction::SignatureE;

/// Specification of multisign. Has multiple keys and specifies which one to use when proposing.
#[derive(Clone)]
pub struct Multi<SecretKey> {
    pub(crate) primary_key_idx: usize,
    pub(crate) keys: Box<[One<SecretKey>]>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct One<SecretKey> {
    pub(crate) key_id: u32,
    pub(crate) key: SecretKey,
}

pub type KeyIdIter<'a, T> = Map<slice::Iter<'a, One<T>>, fn(&'a One<T>) -> u32>;

pub type KeyIter<'a, T> = Map<slice::Iter<'a, One<T>>, fn(&'a One<T>) -> &'a T>;

fn key_id<T>(one: &One<T>) -> u32 {
    one.key_id
}

fn key<T>(one: &One<T>) -> &T {
    &one.key
}

impl<SecretKey> Multi<SecretKey> {
    /// The primary key. Refer to this when proposing a transaction.
    pub fn primary_key(&self) -> &SecretKey {
        &self.keys[self.primary_key_idx].key
    }

    pub fn primary_key_id(&self) -> u32 {
        self.keys[self.primary_key_idx].key_id
    }

    pub fn keys_and_key_ids(&self) -> (KeyIter<SecretKey>, KeyIdIter<SecretKey>) {
        (self.keys(), self.key_ids())
    }

    pub fn key_ids(&self) -> KeyIdIter<SecretKey> {
        self.keys.iter().map(key_id)
    }
    /// Returns an iterator over the secret keys.
    pub fn keys(&self) -> KeyIter<SecretKey> {
        self.keys.iter().map(key)
    }
}

/// How to sign? This describes whether to sign using only one secret key or with multiple keys (multisign).
#[derive(Clone)]
pub enum SignMethod<SecretKey> {
    One(One<SecretKey>),
    Multi(Multi<SecretKey>),
}

impl<SecretKey> SignMethod<SecretKey> {
    /// Returns the primary key used when proposing a transaction.
    pub fn primary_secret_key(&self) -> &SecretKey {
        match self {
            Self::One(one) => &one.key,
            Self::Multi(multi) => multi.primary_key(),
        }
    }

    pub fn primary_key_id(&self) -> u32 {
        match self {
            Self::One(one) => one.key_id,
            Self::Multi(multi) => multi.primary_key_id(),
        }
    }

    pub fn key_ids(&self) -> KeyIdIter<SecretKey> {
        let keys = match self {
            Self::One(one) => unsafe { std::slice::from_raw_parts(one, 1) },
            Self::Multi(multi) => &multi.keys,
        };

        keys.iter().map(key_id)
    }
}

/// Makes `SignatureE`s for encoding.
#[derive(Clone)]
pub struct MkSigIter<'a, KeyIdIter, SigIter> {
    address: &'a [u8],
    iter: Zip<KeyIdIter, SigIter>,
}

impl<'a, KeyIdIter: Iterator<Item = u32>, SigIter: Iterator> MkSigIter<'a, KeyIdIter, SigIter>
where
    SigIter::Item: Signature,
{
    pub fn new(address: &'a [u8], key_ids: KeyIdIter, signatures: SigIter) -> Self {
        Self {
            address,
            iter: key_ids.zip(signatures),
        }
    }
}

impl<'a, KeyIdIter: Iterator<Item = u32>, SigIter: Iterator> Iterator
    for MkSigIter<'a, KeyIdIter, SigIter>
where
    SigIter::Item: Signature,
{
    type Item = SignatureE<&'a [u8], <SigIter::Item as Signature>::Serialized>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key_id, sig)| SignatureE {
            address: self.address,
            signature: sig.serialize(),
            key_id,
        })
    }
}

/// A signature iterator. Iterates over the secret keys and sign the data.
#[derive(Clone)]
pub struct SignIter<'a, Signer: FlowSigner> {
    data: [u8; 32],
    signer: &'a Signer,
    idx: usize,
    method: &'a SignMethod<Signer::SecretKey>,
}

impl<'a, Signer: FlowSigner> SignIter<'a, Signer> {
    /// Creates a new signature iterator with the data being signed, the signer, and the method.
    pub fn new(
        data: [u8; 32],
        signer: &'a Signer,
        method: &'a SignMethod<Signer::SecretKey>,
    ) -> Self {
        Self {
            data,
            signer,
            idx: 0,
            method,
        }
    }

    /// Returns how many signatures are left to sign.
    pub fn remaining(&self) -> usize {
        let count = match self.method {
            SignMethod::One(_) => 1,
            SignMethod::Multi(multi) => multi.keys.len(),
        };

        count - self.idx
    }

    /// Are there more signatures left to sign?
    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    unsafe fn next_unchecked(&mut self) -> Signer::Signature {
        let secret_key = match self.method {
            SignMethod::One(sec) => &sec.key,
            SignMethod::Multi(multi) => &multi.keys.get_unchecked(self.idx).key,
        };

        self.idx += 1;

        self.signer.sign_populated(self.data, secret_key)
    }
}

impl<Signer: FlowSigner> Iterator for SignIter<'_, Signer> {
    type Item = Signer::Signature;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: checked that this iterator has remaining items
        self.has_remaining()
            .then(|| unsafe { self.next_unchecked() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining();
        (len, Some(len))
    }
}

impl<Signer: FlowSigner> FusedIterator for SignIter<'_, Signer> {}

impl<Signer: FlowSigner> ExactSizeIterator for SignIter<'_, Signer> {
    fn len(&self) -> usize {
        self.remaining()
    }
}
