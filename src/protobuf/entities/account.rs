use otopr::{DecodableMessage, Map, Repeated};

#[derive(DecodableMessage, Default)]
pub struct Account {
    pub address: Vec<u8>,
    pub balance: u64,
    pub code: Vec<u8>,
    pub keys: Repeated<Vec<AccountKey>>,
    pub contracts: Map<String, Vec<u8>>,
}

#[derive(DecodableMessage, Default)]
pub struct AccountKey {
    pub index: u32,
    pub public_key: Vec<u8>,
    pub sign_algo: u32,
    pub hash_algo: u32,
    pub weight: u32,
    pub sequence_number: u32,
    pub revoked: bool,
}
