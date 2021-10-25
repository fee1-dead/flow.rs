mod collection;
pub use collection::*;

mod account;
pub use account::*;

mod event;
pub use event::*;

mod execution_result;
pub use execution_result::*;

use crate::protobuf::Timestamp;

use otopr::DecodableMessage;
use otopr::Repeated;

#[derive(DecodableMessage, Default, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    pub id: Vec<u8>,
    pub parent_id: Vec<u8>,
    pub height: u64,
    pub timestamp: Timestamp,
}

#[derive(DecodableMessage, Default, PartialEq, Eq)]
pub struct BlockSeal {
    #[otopr(1)]
    pub block_id: Vec<u8>,
    #[otopr(2)]
    pub execution_receipt_id: Vec<u8>,
    #[otopr(3)]
    pub execution_receipt_signatures: Repeated<Vec<Vec<u8>>>,
    #[otopr(4)]
    pub result_approval_signatures: Repeated<Vec<Vec<u8>>>,
}

#[derive(DecodableMessage, Default, PartialEq, Eq)]
pub struct Block {
    pub id: Vec<u8>,
    pub parent_id: Vec<u8>,
    pub height: u64,
    pub timestamp: Timestamp,
    pub collection_guarantees: Repeated<Vec<CollectionGuarantee>>,
    pub block_seals: Repeated<Vec<BlockSeal>>,
    pub signatures: Repeated<Vec<Vec<u8>>>,
}