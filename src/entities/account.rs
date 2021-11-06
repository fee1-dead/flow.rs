use otopr::{DecodableMessage, Map, Repeated};

/// An account is a user's identity on Flow.
/// It contains a unique address, a balance,
/// a list of public keys and the code that has been deployed to the account.
///
/// The `code` and `contracts` fields contain the raw Cadence source code, encoded as UTF-8 bytes.
///
/// More information on accounts can be found [here](https://docs.onflow.org/concepts/accounts-and-keys/).
#[derive(Clone, DecodableMessage, Default, PartialEq, Eq)]
pub struct Account {
    /// A unique account identifier.
    pub address: Box<[u8]>,

    /// The account balance
    pub balance: u64,

    /// The code deployed to this account (**deprecated**, use contracts instead)
    pub code: Box<[u8]>,

    /// A list of keys configured on this account
    pub keys: Repeated<Vec<AccountKey>>,

    /// A map of contracts or contract interfaces deployed on this account
    pub contracts: Map<String, Box<[u8]>>,
}

/// A key configured on some account.
#[derive(Clone, DecodableMessage, Default, PartialEq, Eq)]
pub struct AccountKey {
    /// The index of the key, also referred as the key ID.
    pub index: u32,

    /// The raw bytes of the public key.
    pub public_key: Box<[u8]>,

    /// The algorithm of which this key uses for signing.
    pub sign_algo: u32,

    /// The algorithm of which this key uses for hashing.
    pub hash_algo: u32,

    /// The weight of this key, with 1000 being the full weight.
    pub weight: u32,

    /// The sequence number of this key.
    pub sequence_number: u32,

    /// Whether this key is revoked.
    pub revoked: bool,
}
