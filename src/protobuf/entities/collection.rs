use otopr::{DecodableMessage, Repeated};

#[derive(DecodableMessage, Default)]
pub struct Collection {
    pub id: Vec<u8>,
    pub transactions: Repeated<Vec<u8>>,
}

#[derive(DecodableMessage, Default)]
pub struct CollectionGuarantee {
    #[otopr(1)]
    pub collection_id: Vec<u8>,
    #[otopr(2)]
    pub signatures: Repeated<Vec<u8>>,
}
