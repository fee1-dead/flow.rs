use otopr::{DecodableMessage, Repeated};

/// A collection is a batch of transactions that have been included in a block.
///
/// Collections are used to improve consensus throughput by increasing the number of transactions per block.
#[derive(DecodableMessage, Default)]
pub struct Collection {
    /// SHA3-256 hash of the collection contents
    pub id: Box<[u8]>,

    /// Ordered list of transaction IDs in the collection
    pub transactions: Repeated<Vec<Box<[u8]>>>,
}

/// A collection guarantee is a signed attestation that specifies the collection nodes that have guaranteed to
/// store and respond to queries about a collection.
#[derive(DecodableMessage, Default, PartialEq, Eq)]
pub struct CollectionGuarantee {
    /// SHA3-256 hash of the collection contents
    pub collection_id: Box<[u8]>,

    /// BLS signatures of the collection nodes guaranteeing the collection
    pub signatures: Repeated<Vec<Box<[u8]>>>,
}
