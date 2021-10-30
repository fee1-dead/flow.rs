//! A trait to generalize requests and responses of the Access API.

use crate::access::*;

mod private {
    pub trait Sealed {}
}

/// A callable request of the Flow Access API.
pub trait FlowRequest<Response>: private::Sealed {
    /// The path of the request.
    ///
    /// formatted as "/"({package} ".")? {service}"/" {method}.
    const PATH: &'static str;
}

macro_rules! access_api {
    (rpc $servName:ident$(<$($generics:ident),+>)?(noseal $reqTy:ty) returns ($resTy:ty) $(where($($tt:tt)*))?) => {
        impl$(<$($generics),+>)? FlowRequest<$resTy> for $reqTy $(where $($tt)*)? {
            const PATH: &'static str = concat!("/flow.access.AccessAPI/", stringify!($servName));
        }
    };
    (rpc $servName:ident$(<$($generics:ident),+>)?($reqTy:ty) returns ($resTy:ty) $(where($($tt:tt)*))?) => {
        access_api!(rpc $servName$(<$($generics),+>)?(noseal $reqTy) returns ($resTy) $(where($($tt)*))?);

        impl$(<$($generics),+>)? private::Sealed for $reqTy $(where $($tt)*)? {}
    };
    ($(rpc $servName:ident$(<$($generics:ident),+$(,)?>)?($($tt:tt)*) returns ($resTy:ty) $(where($($tts:tt)*))?;)+) => {
        $(
            access_api!(rpc $servName$(<$($generics),+>)?($($tt)*) returns ($resTy) $(where($($tts)*))?);
        )+
    };
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
    >) returns (SendTransactionResponse);
    rpc GetTransaction(GetTransactionRequest<'_>) returns (TransactionResponse);
    rpc GetTransactionResult(noseal GetTransactionRequest<'_>)
        returns (TransactionResultResponse);
    rpc GetAccountAtLatestBlock(GetAccountAtLatestBlockRequest<'_>)
        returns (AccountResponse);
    rpc GetAccountAtBlockHeight(GetAccountAtBlockHeightRequest<'_>)
        returns (AccountResponse);
    rpc ExecuteScriptAtLatestBlock<Script, Arguments>(ExecuteScriptAtLatestBlockRequest<Script, Arguments>)
        returns (ExecuteScriptResponse);
    rpc ExecuteScriptAtBlockID<BlockId, Script, Arguments>(ExecuteScriptAtBlockIdRequest<BlockId, Script, Arguments>)
        returns (ExecuteScriptResponse);
    rpc ExecuteScriptAtBlockHeight<Script, Arguments>(ExecuteScriptAtBlockHeightRequest<Script, Arguments>)
        returns (ExecuteScriptResponse);
    rpc GetEventsForHeightRange<EventTy>(GetEventsForHeightRangeRequest<EventTy>)
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
