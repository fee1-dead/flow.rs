//! Entities of the Flow network.
//!
//! This module contains data entities returned or accepted by the Access API.

mod collection;
pub use collection::*;

mod account;
pub use account::*;

mod event;
pub use event::*;

mod execution_result;
pub use execution_result::*;
use otopr::{DecodableMessage, Repeated};

use crate::protobuf::Timestamp;

/// A block header is a summary of a block and contains only the block ID, height, and parent block ID.
#[derive(DecodableMessage, Default, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    /// SHA3-256 hash of the entire block payload
    pub id: Box<[u8]>,

    /// ID of the previous block in the chain
    pub parent_id: Box<[u8]>,

    /// Height of the block in the chain
    pub height: u64,

    /// Timestamp of when the proposer claims it constructed the block.
    ///
    /// NOTE: It is included by the proposer,
    /// there are no guarantees on how much the time stamp can deviate from the true time the block was published.
    /// Consider observing blocks' status changes yourself to get a more reliable value.
    pub timestamp: Timestamp,
}

/// A block seal is an attestation that the execution result of a specific [`Block`] has been verified and approved
/// by a quorum of verification nodes.
#[derive(DecodableMessage, Default, PartialEq, Eq)]
pub struct BlockSeal {
    /// ID of the block being sealed
    pub block_id: Box<[u8]>,

    /// ID of the execution receipt being sealed
    pub execution_receipt_id: Box<[u8]>,

    /// BLS signatures of verification nodes on the execution receipt contents
    pub execution_receipt_signatures: Repeated<Vec<Box<[u8]>>>,

    /// BLS signatures of verification nodes on the result approval contents
    pub result_approval_signatures: Repeated<Vec<Box<[u8]>>>,
}

/// A block.
#[derive(DecodableMessage, Default, PartialEq, Eq)]
pub struct Block {
    /// SHA3-256 hash of the entire block payload
    pub id: Box<[u8]>,

    /// ID of the previous block in the chain
    pub parent_id: Box<[u8]>,

    /// Height of the block in the chain
    pub height: u64,

    /// Timestamp of when the proposer claims it constructed the block.
    ///
    /// NOTE: It is included by the proposer,
    /// there are no guarantees on how much the time stamp can deviate from the true time the block was published.
    /// Consider observing blocks' status changes yourself to get a more reliable value.
    pub timestamp: Timestamp,

    /// List of [collection guarantees](CollectionGuarantee).
    pub collection_guarantees: Repeated<Vec<CollectionGuarantee>>,

    /// List of [block seals](BlockSeal).
    pub block_seals: Repeated<Vec<BlockSeal>>,

    /// BLS signatures of consensus nodes
    pub signatures: Repeated<Vec<Box<[u8]>>>,
}
