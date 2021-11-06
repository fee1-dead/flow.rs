//! Access API type definitions
//!
//! This modules contains definitions for requests and responses of the access API.

use std::time::Duration;

use otopr::wire_types::*;
use otopr::*;

use crate::client::GrpcClient;
use crate::entities::*;
use crate::protobuf::*;
use crate::trait_hack::Hack;
use crate::transaction::*;

/// Ping.
#[derive(EncodableMessage)]
pub struct PingRequest;

/// Pong.
#[derive(DecodableMessage, Default)]
pub struct PingResponse;

/// A block header.
#[derive(DecodableMessage, Default)]
pub struct BlockHeaderResponse(pub BlockHeader);

/// Gets the latest block's header.
#[derive(EncodableMessage)]
pub struct GetLatestBlockHeaderRequest {
    /// Whether the response should be sealed.
    pub seal: Seal,
}

/// Gets a block's header by id.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Id: AsRef<[u8]>))]
pub struct GetBlockHeaderByIdRequest<Id> {
    /// id of the block.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub id: Id,
}

/// Gets a block's header by its height.
#[derive(EncodableMessage)]
pub struct GetBlockHeaderByHeightRequest {
    /// height of the block.
    #[otopr(1)]
    pub height: u64,
}

/// Full information about a block.
#[derive(DecodableMessage, Default)]
pub struct BlockResponse(pub Block);

/// Gets full information about the latest block.
#[derive(EncodableMessage)]
pub struct GetLatestBlockRequest {
    /// Whether to get latest "sealed" block or any latest block.
    pub seal: Seal,
}

/// Gets full information about a block by its id.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Id: AsRef<[u8]>))]
pub struct GetBlockByIdRequest<Id> {
    /// id of the block.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub id: Id,
}

/// Gets full information about a block by its height.
#[derive(EncodableMessage)]
pub struct GetBlockByHeightRequest {
    /// height of the block.
    pub height: u64,
}

/// Gets information about a collection by its id.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Id: AsRef<[u8]>))]
pub struct GetCollectionByIdRequest<Id> {
    /// id of the collection.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub id: Id,
}

/// A collection.
#[derive(DecodableMessage, Default)]
pub struct CollectionResponse {
    /// The collection.
    pub collection: Collection,
}

#[derive(EncodableMessage)]
#[otopr(encode_extra_type_params(
    PayloadSignatureAddress,
    PayloadSignature,
    EnvelopeSignatureAddress,
    EnvelopeSignature,
))]
#[otopr(encode_where_clause(
    where
        Script: AsRef<[u8]>,
        ReferenceBlockId: AsRef<[u8]>,
        Payer: AsRef<[u8]>,
        ProposalKeyAddress: AsRef<[u8]>,
        PayloadSignatureAddress: AsRef<[u8]>,
        PayloadSignature: AsRef<[u8]>,
        EnvelopeSignatureAddress: AsRef<[u8]>,
        EnvelopeSignature: AsRef<[u8]>,
        Arguments: HasItem,
        <Arguments as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Arguments: IntoIterator<Item = &'a <Arguments as HasItem>::Item>,
        for<'a> Hack<<&'a Arguments as IntoIterator>::IntoIter>: Clone,
        Authorizers: HasItem,
        <Authorizers as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Authorizers: IntoIterator<Item = &'a <Authorizers as HasItem>::Item>,
        for<'a> Hack<<&'a Authorizers as IntoIterator>::IntoIter>: Clone,
        PayloadSignatures: HasItem<Item = SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> &'a PayloadSignatures: IntoIterator<Item = &'a SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> Hack<<&'a PayloadSignatures as IntoIterator>::IntoIter>: Clone,
        EnvelopeSignatures: HasItem<Item = SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> &'a EnvelopeSignatures: IntoIterator<Item = &'a SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> Hack<<&'a EnvelopeSignatures as IntoIterator>::IntoIter>: Clone,
))]
/// Sends a transaction over the network.
pub struct SendTransactionRequest<
    Script,
    Arguments,
    ReferenceBlockId,
    ProposalKeyAddress,
    Payer,
    Authorizers,
    PayloadSignatures,
    EnvelopeSignatures,
> {
    /// The transaction to be sent.
    pub transaction: TransactionE<
        Script,
        Arguments,
        ReferenceBlockId,
        ProposalKeyAddress,
        Payer,
        Authorizers,
        PayloadSignatures,
        EnvelopeSignatures,
    >,
}

/// The id of the transaction on the network.
#[derive(DecodableMessage, Default)]
pub struct SendTransactionResponse {
    /// The id of the transaction.
    pub id: Vec<u8>,
}

impl SendTransactionResponse {
    /// Returns a future that periodically queries the transaction response until the transaction is sealed or expired.
    ///
    /// The default delay is 2 seconds between requests, and the default timeout is 15 seconds.
    ///
    /// To customize the delay and the timeout, refer to [`Finalize`]'s documentation.
    ///
    /// [`Finalize`]: crate::transaction::Finalize
    pub fn finalize<'a, C: GrpcClient<GetTransactionRequest<&'a [u8]>, TransactionResultResponse>>(
        &'a self,
        client: C,
    ) -> Finalize<&'a [u8], C> {
        Finalize::new(
            &self.id,
            client,
            Duration::from_secs(2),
            Duration::from_secs(60),
        )
    }
}

/// Gets a transaction's details by its id.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Id: AsRef<[u8]>))]
pub struct GetTransactionRequest<Id> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// the id of the transaction.
    pub id: Id,
}

/// The full details of a transaction.
#[derive(DecodableMessage, Default)]
pub struct TransactionResponse {
    /// The transaction.
    pub transaction: TransactionD,
}

/// The results of a transaction.
#[derive(DecodableMessage, Default)]
pub struct TransactionResultResponse {
    /// The status of the transaction.
    pub status: TransactionStatus,
    /// The status code of the transaction.
    pub status_code: u32,
    /// The error message, if any.
    pub error_message: String,
    /// The events of the transaction.
    pub events: Repeated<Vec<Event>>,
    /// The block ID of the transaction.
    pub block_id: Box<[u8]>,
}

/// Retrieves information of an account at the latest block.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Address: AsRef<[u8]>))]
pub struct GetAccountAtLatestBlockRequest<Address> {
    /// The raw bytes of the address of the account.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub address: Address,
}

/// Retrieves information of an account at the specific block height.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Address: AsRef<[u8]>))]
pub struct GetAccountAtBlockHeightRequest<Address> {
    /// The raw bytes of the address of the account.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub address: Address,
    /// The block height.
    pub block_height: u64,
}

/// An account.
#[derive(DecodableMessage, Default)]
pub struct AccountResponse {
    /// The account.
    pub account: Account,
}

fn encode_argument<'a, T: serde::Serialize + 'a, It: Iterator<Item = &'a T> + 'a>(
    it: It,
) -> std::iter::Map<It, fn(&T) -> Vec<u8>> {
    fn enc<T: serde::Serialize>(t: &T) -> Vec<u8> {
        serde_json::to_vec(t).unwrap()
    }
    it.map(enc)
}

#[derive(EncodableMessage)]
#[otopr(encode_where_clause(
    where
        Script: AsRef<[u8]>,
        Arguments: HasItem,
        <Arguments as HasItem>::Item: serde::Serialize,
        for<'a> &'a Arguments: IntoIterator<Item = &'a <Arguments as HasItem>::Item>,
        for<'a> Hack<<&'a Arguments as IntoIterator>::IntoIter>: Clone,
))]
/// Executes a script (maybe with arguments) at the latest block.
pub struct ExecuteScriptAtLatestBlockRequest<Script, Arguments> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// The script.
    pub script: Script,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |hacked| encode_argument(hacked.0))))]
    /// The arguments. A collection of [`ValueRef`]s.
    ///
    /// [`ValueRef`]: cadence_json::ValueRef
    pub arguments: Arguments,
}

/// Executes a script (maybe with arguments) at a block specified by its block ID.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(
    where
        BlockId: AsRef<[u8]>,
        Script: AsRef<[u8]>,
        Arguments: HasItem,
        <Arguments as HasItem>::Item: serde::Serialize,
        for<'a> &'a Arguments: IntoIterator<Item = &'a <Arguments as HasItem>::Item>,
        for<'a> Hack<<&'a Arguments as IntoIterator>::IntoIter>: Clone,
))]
pub struct ExecuteScriptAtBlockIdRequest<BlockId, Script, Arguments> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// id of the block.
    pub block_id: BlockId,

    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// The script.
    pub script: Script,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |hacked| encode_argument(hacked.0))))]
    /// The arguments. A collection of [`ValueRef`]s.
    ///
    /// [`ValueRef`]: cadence_json::ValueRef
    pub arguments: Arguments,
}

/// Executes a script (maybe with arguments) at a block specified by its height.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(
    where
        Script: AsRef<[u8]>,
        Arguments: HasItem,
        <Arguments as HasItem>::Item: serde::Serialize,
        for<'a> &'a Arguments: IntoIterator<Item = &'a <Arguments as HasItem>::Item>,
        for<'a> Hack<<&'a Arguments as IntoIterator>::IntoIter>: Clone,
))]
pub struct ExecuteScriptAtBlockHeightRequest<Script, Arguments> {
    /// height of the block.
    pub block_height: u64,

    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// The script.
    pub script: Script,

    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |hacked| encode_argument(hacked.0))))]
    /// The arguments. A collection of [`ValueRef`]s.
    ///
    /// [`ValueRef`]: cadence_json::ValueRef
    pub arguments: Arguments,
}

/// The return value of the script.
#[derive(DecodableMessage, Default)]
pub struct ExecuteScriptResponse {
    /// The return value. Use [`ExecuteScriptResponse::parse()`] to parse it.
    pub value: Box<[u8]>,
}

impl ExecuteScriptResponse {
    /// Parses the value.
    pub fn parse(&self) -> serde_json::Result<cadence_json::ValueOwned> {
        serde_json::from_slice(&self.value)
    }
}

/// Search for events in a height range.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(
    where
        EventTy: AsRef<str>,
))]
pub struct GetEventsForHeightRangeRequest<EventTy> {
    /// The type of the event we are looking for.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub ty: EventTy,
    /// The start of the height range.
    pub start_height: u64,
    /// The end of the height range.
    pub end_height: u64,
}

/// Search for events in a collection of blocks specified by its block id.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(
    where
        EventTy: AsRef<str>,
        BlockIds: HasItem,
        <BlockIds as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a BlockIds: IntoIterator<Item = &'a <BlockIds as HasItem>::Item>,
        for<'a> Hack<<&'a BlockIds as IntoIterator>::IntoIter>: Clone,
))]
pub struct GetEventsForBlockIdsRequest<EventTy, BlockIds> {
    /// The type of the event we are looking for.
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    pub ty: EventTy,
    /// The IDs of the blocks.
    #[otopr(encode_via(wire_types::LengthDelimitedWire, RepeatedMap::new(Hack(x.into_iter()), |it| it.0.map(AsRef::as_ref))))]
    pub block_ids: BlockIds,
}

/// Search results for events in a single block.
#[derive(DecodableMessage, Default)]
pub struct EventsResult {
    /// The ID of the block.
    pub block_id: Box<[u8]>,
    /// The height of the block.
    pub block_height: u64,
    /// The events that occured on this block.
    pub events: Repeated<Vec<Event>>,
    /// The timestamp of the block.
    pub block_timestamp: Timestamp,
}

/// Search results for events in multiple blocks.
#[derive(DecodableMessage, Default)]
pub struct EventsResponse {
    /// The results.
    pub results: Repeated<Vec<EventsResult>>,
}

/// Get network parameters.
#[derive(EncodableMessage)]
pub struct GetNetworkParametersRequest;

/// The network parameters.
#[derive(DecodableMessage, Default)]
pub struct GetNetworkParametersResponse {
    /// The chain ID.
    pub chain_id: String,
}

/// Retrieves the latest Protocol state snapshot serialized as a byte array.
///
/// It is used by Flow nodes joining the network to bootstrap a space-efficient local state.
#[derive(EncodableMessage)]
pub struct GetLatestProtocolStateSnapshotRequest;

/// A protocol state snapshot.
#[derive(DecodableMessage, Default)]
pub struct ProtocolStateSnapshotResponse {
    /// The serialized snapshop.
    pub serialized_snapshot: Box<[u8]>,
}

/// Retrieves execution result for given block. It is different from Transaction Results,
/// and contain data about chunks/collection level execution results rather than particular transactions.
/// Particularly, it contains EventsCollection hash for every chunk which can be used to verify the events for a block.
#[derive(EncodableMessage)]
#[otopr(encode_where_clause(where Id: AsRef<[u8]>))]
pub struct GetExecutionResultForBlockIdRequest<Id> {
    #[otopr(encode_via(LengthDelimitedWire, x.as_ref()))]
    /// ID of the block.
    pub block_id: Id,
}

/// An execution result.
#[derive(DecodableMessage, Default)]
pub struct ExecutionResultForBlockIdResponse {
    /// The execution result.
    pub execution_result: ExecutionResult,
}
