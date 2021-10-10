use crate::{ExecutionResult, Timestamp};
use otopr::{DecodableMessage, EncodableMessage, Message, Repeated};

macro_rules! access_api {
    (rpc $servName:ident(noseal $reqTy:ty) returns ($resTy:ty)) => {
        impl FlowRequest<$resTy> for $reqTy {
            const PATH: &'static str = concat!("/flow.access.AccessAPI/", stringify!($servName));
        }
    };
    (rpc $servName:ident($reqTy:ty) returns ($resTy:ty)) => {
        access_api!(rpc $servName(noseal $reqTy) returns ($resTy));

        impl crate::requests::private::Sealed for $reqTy {}
    };
    ($(rpc $servName:ident($($tt:tt)*) returns ($resTy:ty);)+) => {
        $(
            access_api!(rpc $servName($($tt)*) returns ($resTy));
        )+
    };
}

use crate::{
    requests::FlowRequest, Account, Collection, Event, RepSlice, TransactionD, TransactionE,
    TransactionStatus,
};

#[derive(EncodableMessage)]
pub struct PingRequest;

#[derive(DecodableMessage, Default)]
pub struct PingResponse;

#[derive(DecodableMessage, Default)]
pub struct BlockHeaderResponse(#[otopr(1)] pub Message<super::BlockHeader>);

#[derive(EncodableMessage)]
pub struct GetLatestBlockHeaderRequest {
    #[otopr(1)]
    pub is_sealed: bool,
}

#[derive(EncodableMessage)]
pub struct GetBlockHeaderByIdRequest<'a> {
    #[otopr(1)]
    pub id: &'a [u8],
}

#[derive(EncodableMessage)]
pub struct GetBlockHeaderByHeightRequest {
    #[otopr(1)]
    pub height: u64,
}

#[derive(DecodableMessage, Default)]
pub struct BlockResponse(#[otopr(1)] pub Message<super::Block>);

#[derive(EncodableMessage)]
pub struct GetLatestBlockRequest {
    #[otopr(1)]
    pub is_sealed: bool,
}

#[derive(EncodableMessage)]
pub struct GetBlockByIdRequest<'a> {
    #[otopr(1)]
    pub id: &'a [u8],
}

#[derive(EncodableMessage)]
pub struct GetBlockByHeightRequest {
    #[otopr(1)]
    pub height: u64,
}

#[derive(EncodableMessage)]
pub struct GetCollectionByIdRequest<'a> {
    pub id: &'a [u8],
}

#[derive(DecodableMessage, Default)]
pub struct CollectionResponse {
    pub collection: Collection,
}

#[derive(EncodableMessage)]
pub struct SendTransactionRequest<'a> {
    pub transaction: TransactionE<'a>,
}

#[derive(DecodableMessage, Default)]
pub struct SendTransactionResponse {
    pub id: Vec<u8>,
}

#[derive(EncodableMessage)]
pub struct GetTransactionRequest<'a> {
    pub id: &'a [u8],
}

#[derive(DecodableMessage, Default)]
pub struct TransactionResponse {
    pub transaction: TransactionD,
}

#[derive(DecodableMessage, Default)]
pub struct TransactionResultResponse {
    pub status: TransactionStatus,
    pub status_code: u32,
    pub error_message: String,
    pub events: Repeated<Event>,
    pub block_id: Vec<u8>,
}

#[derive(EncodableMessage)]
pub struct GetAccountAtLatestBlockRequest<'a> {
    pub id: &'a [u8],
}

#[derive(EncodableMessage)]
pub struct GetAccountAtBlockHeightRequest<'a> {
    pub id: &'a [u8],
    pub block_height: u64,
}

#[derive(DecodableMessage, Default)]
pub struct AccountResponse {
    pub account: Account,
}

#[derive(EncodableMessage)]
pub struct ExecuteScriptAtLatestBlockRequest<'a> {
    pub script: &'a [u8],
    pub arguments: RepSlice<'a, &'a [u8]>,
}

#[derive(EncodableMessage)]
pub struct ExecuteScriptAtBlockIdRequest<'a> {
    pub block_id: &'a [u8],
    pub script: &'a [u8],
    pub arguments: RepSlice<'a, &'a [u8]>,
}

#[derive(EncodableMessage)]
pub struct ExecuteScriptAtBlockHeightRequest<'a> {
    pub block_height: u64,
    pub script: &'a [u8],
    pub arguments: RepSlice<'a, &'a [u8]>,
}

#[derive(DecodableMessage, Default)]
pub struct ExecuteScriptResponse {
    pub value: Vec<u8>,
}

#[derive(EncodableMessage)]
pub struct GetEventsForHeightRangeRequest<'a> {
    pub r#type: &'a str,
    pub start_height: u64,
    pub end_height: u64,
}

#[derive(EncodableMessage)]
pub struct GetEventsForBlockIdsRequest<'a> {
    pub r#type: &'a str,
    pub block_ids: RepSlice<'a, &'a [u8]>,
}

#[derive(DecodableMessage, Default)]
pub struct EventsResult {
    pub block_id: Vec<u8>,
    pub block_height: u64,
    pub events: Repeated<Event>,
    pub block_timestamp: Timestamp,
}

#[derive(DecodableMessage, Default)]
pub struct EventsResponse {
    pub results: Repeated<EventsResult>,
}

#[derive(EncodableMessage)]
pub struct GetNetworkParametersRequest;

#[derive(DecodableMessage, Default)]
pub struct GetNetworkParametersResponse {
    pub chain_id: String,
}

#[derive(EncodableMessage)]
pub struct GetLatestProtocolStateSnapshotRequest;

#[derive(DecodableMessage, Default)]
pub struct ProtocolStateSnapshotResponse {
    pub serialized_snapshot: Vec<u8>,
}

#[derive(EncodableMessage)]
pub struct GetExecutionResultForBlockIdRequest<'a> {
    pub block_id: &'a [u8],
}

#[derive(DecodableMessage, Default)]
pub struct ExecutionResultForBlockIdResponse {
    pub execution_result: ExecutionResult,
}

access_api! {
    rpc Ping(PingRequest) returns (PingResponse);
    rpc GetLatestBlockHeader(GetLatestBlockHeaderRequest)
        returns (BlockHeaderResponse);
    rpc GetBlockHeaderByID(GetBlockHeaderByIdRequest<'_>)
        returns (BlockHeaderResponse);
    rpc GetBlockHeaderByHeight(GetBlockHeaderByHeightRequest)
        returns (BlockHeaderResponse);
    rpc GetLatestBlock(GetLatestBlockRequest) returns (BlockResponse);
    rpc GetBlockByID(GetBlockByIdRequest<'_>) returns (BlockResponse);
    rpc GetBlockByHeight(GetBlockByHeightRequest) returns (BlockResponse);
    rpc GetCollectionByID(GetCollectionByIdRequest<'_>) returns (CollectionResponse);
    rpc SendTransaction(SendTransactionRequest<'_>) returns (SendTransactionResponse);
    rpc GetTransaction(GetTransactionRequest<'_>) returns (TransactionResponse);
    rpc GetTransactionResult(noseal GetTransactionRequest<'_>)
        returns (TransactionResultResponse);
    rpc GetAccountAtLatestBlock(GetAccountAtLatestBlockRequest<'_>)
        returns (AccountResponse);
    rpc GetAccountAtBlockHeight(GetAccountAtBlockHeightRequest<'_>)
        returns (AccountResponse);
    rpc ExecuteScriptAtLatestBlock(ExecuteScriptAtLatestBlockRequest<'_>)
        returns (ExecuteScriptResponse);
    rpc ExecuteScriptAtBlockID(ExecuteScriptAtBlockIdRequest<'_>)
        returns (ExecuteScriptResponse);
    rpc ExecuteScriptAtBlockHeight(ExecuteScriptAtBlockHeightRequest<'_>)
        returns (ExecuteScriptResponse);
    rpc GetEventsForHeightRange(GetEventsForHeightRangeRequest<'_>)
        returns (EventsResponse);
    rpc GetEventsForBlockIDs(GetEventsForBlockIdsRequest<'_>)
        returns (EventsResponse);
    rpc GetNetworkParameters(GetNetworkParametersRequest)
        returns (GetNetworkParametersResponse);
    rpc GetLatestProtocolStateSnapshot(GetLatestProtocolStateSnapshotRequest)
        returns (ProtocolStateSnapshotResponse);
    rpc GetExecutionResultForBlockID(GetExecutionResultForBlockIdRequest<'_>)
        returns (ExecutionResultForBlockIdResponse);
}
