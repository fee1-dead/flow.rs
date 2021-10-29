use otopr::*;

/// Execution result for a particular block.
#[derive(DecodableMessage, Default)]
pub struct ExecutionResult {
    /// Identifier of parent block execution result.
    pub previous_result_id: Box<[u8]>,
    /// ID of the block this execution result corresponds to.
    pub block_id: Box<[u8]>,
    /// Chunks within this execution.
    pub chunks: Repeated<Vec<Chunk>>,
    /// Service events that occured within this execution.
    pub service_events: Repeated<Vec<ServiceEvent>>,
}

/// Chunk describes execution information for given collection in a block.
#[derive(DecodableMessage, Default)]
pub struct Chunk {
    /// State commitment at start of the chunk.
    pub start_state: Box<[u8]>,
    /// Hash of events emitted by transactions in this chunk.
    pub event_collection: Box<[u8]>,
    /// Block id of the execution result this chunk belongs to.
    pub block_id: Box<[u8]>,
    /// Total amount of computation used by running all Transactions in this chunk.
    pub total_computation_used: u64,
    /// Number of transactions inside the chunk.
    pub number_of_transactions: u64,
    /// Index of chunk inside a block (zero-based)
    pub index: u64,
    /// State commitment after executing chunk
    pub end_state: Box<[u8]>,
}

/// Special type of events emitted in system chunk used for controlling Flow system.
#[derive(DecodableMessage, Default)]
pub struct ServiceEvent {
    /// Type of an event
    pub ty: String,
    /// JSON-serialized content of an event.
    pub payload: Vec<u8>,
}
