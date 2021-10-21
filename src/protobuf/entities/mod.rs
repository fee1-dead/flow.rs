use std::fmt;

mod collection;
pub use collection::*;

mod account;
pub use account::*;

mod event;
pub use event::*;

mod execution_result;
pub use execution_result::*;

use otopr::DecodableMessage;
use otopr::Message;
use otopr::Repeated;

#[derive(DecodableMessage, Default, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    #[otopr(1)]
    pub id: Vec<u8>,
    #[otopr(2)]
    pub parent_id: Vec<u8>,
    #[otopr(3)]
    pub height: u64,
    #[otopr(4)]
    pub timestamp: Message<super::Timestamp>,
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
    #[otopr(1)]
    pub id: Vec<u8>,
    #[otopr(2)]
    pub parent_id: Vec<u8>,
    #[otopr(3)]
    pub height: u64,
    #[otopr(4)]
    pub timestamp: super::Timestamp,
    #[otopr(5)]
    pub collection_guarantees: Repeated<Vec<CollectionGuarantee>>,
    #[otopr(6)]
    pub block_seals: Repeated<Vec<BlockSeal>>,
    #[otopr(7)]
    pub signatures: Repeated<Vec<Vec<u8>>>,
}

struct IdDebug<'a>(&'a Vec<Vec<u8>>);

impl fmt::Debug for IdDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(hex::encode))
            .finish()
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("id", &hex::encode(&self.id))
            .field("parent_id", &hex::encode(&self.parent_id))
            .field("height", &self.height)
            .field("timestamp", &self.timestamp)
            .field("collection_guarantees", &self.collection_guarantees)
            .field("block_seals", &self.block_seals)
            .field("signatures", &IdDebug(&self.signatures))
            .finish()
    }
}

impl fmt::Debug for BlockSeal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockSeal")
            .field("block_id", &hex::encode(&self.block_id))
            .field(
                "execution_receipt_id",
                &hex::encode(&self.execution_receipt_id),
            )
            .field(
                "execution_receipt_signatures",
                &IdDebug(&self.execution_receipt_signatures),
            )
            .field(
                "result_approval_signatures",
                &IdDebug(&self.result_approval_signatures),
            )
            .finish()
    }
}
