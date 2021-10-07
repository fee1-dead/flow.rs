use otopr::{DecodableMessage, EncodableMessage, Message};

macro_rules! access_api {
    ($(rpc $servName:ident($reqTy:ty) returns ($resTy:ty);)+) => {
        $(
            impl FlowRequest<$resTy> for $reqTy {
                const PATH: &'static str = concat!("/flow.access.AccessAPI/", stringify!($servName));
            }
            impl crate::requests::private::Sealed for $reqTy {}
        )+
    };
}

use crate::requests::FlowRequest;

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

access_api! {
    rpc Ping(PingRequest) returns (PingResponse);
    rpc GetLatestBlockHeader(GetLatestBlockHeaderRequest)
        returns (BlockHeaderResponse);
    rpc GetBlockHeaderByID(GetBlockHeaderByIdRequest<'_>)
        returns (BlockHeaderResponse);
    rpc GetBlockHeaderByHeight(GetBlockHeaderByHeightRequest)
        returns (BlockHeaderResponse);
}
