use std::time::Duration;

use crate::{
    client::{FlowClient, GrpcClient},
    ExecutionResult, Finalize, SignatureE, Timestamp,
};
use otopr::*;

macro_rules! access_api {
    (rpc $servName:ident$(<$($generics:ident),+>)?(noseal $reqTy:ty) returns ($resTy:ty) $(where($($tt:tt)*))?) => {
        impl$(<$($generics),+>)? FlowRequest<$resTy> for $reqTy $(where $($tt)*)? {
            const PATH: &'static str = concat!("/flow.access.AccessAPI/", stringify!($servName));
        }
    };
    (rpc $servName:ident$(<$($generics:ident),+>)?($reqTy:ty) returns ($resTy:ty) $(where($($tt:tt)*))?) => {
        access_api!(rpc $servName$(<$($generics),+>)?(noseal $reqTy) returns ($resTy) $(where($($tt)*))?);

        impl$(<$($generics),+>)? crate::requests::private::Sealed for $reqTy $(where $($tt)*)? {}
    };
    ($(rpc $servName:ident$(<$($generics:ident),+$(,)?>)?($($tt:tt)*) returns ($resTy:ty) $(where($($tts:tt)*))?;)+) => {
        $(
            access_api!(rpc $servName$(<$($generics),+>)?($($tt)*) returns ($resTy) $(where($($tts)*))?);
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
pub struct BlockHeaderResponse(#[otopr(1)] pub super::BlockHeader);

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
pub struct BlockResponse(pub super::Block);

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
        for<'a> <&'a Arguments as IntoIterator>::IntoIter: Clone,
        Authorizers: HasItem,
        <Authorizers as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Authorizers: IntoIterator<Item = &'a <Authorizers as HasItem>::Item>,
        for<'a> <&'a Authorizers as IntoIterator>::IntoIter: Clone,
        PayloadSignatures: HasItem<Item = SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> &'a PayloadSignatures: IntoIterator<Item = &'a SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> <&'a PayloadSignatures as IntoIterator>::IntoIter: Clone,
        EnvelopeSignatures: HasItem<Item = SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> &'a EnvelopeSignatures: IntoIterator<Item = &'a SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> <&'a EnvelopeSignatures as IntoIterator>::IntoIter: Clone,
))]
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

#[derive(DecodableMessage, Default)]
pub struct SendTransactionResponse {
    pub id: Vec<u8>,
}

impl SendTransactionResponse {
    /// Returns a future that periodically queries the transaction response until the transaction is sealed or expired.
    ///
    /// The default delay is 2 seconds between requests, and the default timeout is 15 seconds.
    pub fn finalize<'a, C: GrpcClient<GetTransactionRequest<'a>, TransactionResultResponse>>(
        &'a self,
        client: &'a mut FlowClient<C>,
    ) -> Finalize<'a, C> {
        Finalize::new(
            &self.id,
            client,
            Duration::from_secs(2),
            Duration::from_secs(15),
        )
    }
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
    pub events: Repeated<Vec<Event>>,
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

impl ExecuteScriptResponse {
    pub fn parse(&self) -> serde_json::Result<cadence_json::ValueOwned> {
        serde_json::from_slice(&self.value)
    }
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
    pub events: Repeated<Vec<Event>>,
    pub block_timestamp: Timestamp,
}

#[derive(DecodableMessage, Default)]
pub struct EventsResponse {
    pub results: Repeated<Vec<EventsResult>>,
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
    rpc SendTransaction<
        PayloadSignatureAddress,
        PayloadSignature,
        EnvelopeSignatureAddress,
        EnvelopeSignature,
        Script,
        Arguments,
        ReferenceBlockId,
        ProposalKeyAddress,
        Payer,
        Authorizers,
        PayloadSignatures,
        EnvelopeSignatures,
    >(SendTransactionRequest<
        Script,
        Arguments,
        ReferenceBlockId,
        ProposalKeyAddress,
        Payer,
        Authorizers,
        PayloadSignatures,
        EnvelopeSignatures,
    >) returns (SendTransactionResponse) where (
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
        for<'a> <&'a Arguments as IntoIterator>::IntoIter: Clone,
        Authorizers: HasItem,
        <Authorizers as HasItem>::Item: AsRef<[u8]>,
        for<'a> &'a Authorizers: IntoIterator<Item = &'a <Authorizers as HasItem>::Item>,
        for<'a> <&'a Authorizers as IntoIterator>::IntoIter: Clone,
        PayloadSignatures: HasItem<Item = SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> &'a PayloadSignatures: IntoIterator<Item = &'a SignatureE<PayloadSignatureAddress, PayloadSignature>>,
        for<'a> <&'a PayloadSignatures as IntoIterator>::IntoIter: Clone,
        EnvelopeSignatures: HasItem<Item = SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> &'a EnvelopeSignatures: IntoIterator<Item = &'a SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
        for<'a> <&'a EnvelopeSignatures as IntoIterator>::IntoIter: Clone,
    );
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
