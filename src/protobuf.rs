
pub mod access {
    use otopr::{EncodableMessage, DecodableMessage, Message,};

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

    #[derive(Clone, Copy, Default, DecodableMessage, Debug)]
    pub struct Timestamp {
        #[otopr(1)]
        pub seconds: i64,
        #[otopr(2)]
        pub nanos: i32,
    }

    #[derive(EncodableMessage)]
    pub struct PingRequest;

    #[derive(DecodableMessage, Default)]
    pub struct PingResponse;

    #[derive(DecodableMessage, Default, Debug)]
    pub struct BlockHeader {
        #[otopr(1)]
        pub id: Vec<u8>,
        #[otopr(2)]
        pub parent_id: Vec<u8>,
        #[otopr(3)]
        pub height: u64,
        #[otopr(4)]
        pub timestamp: Message<Timestamp>,
    }

    #[derive(DecodableMessage, Default)]
    pub struct BlockHeaderResponse(#[otopr(1)] pub Message<BlockHeader>);

    #[derive(EncodableMessage)]
    pub struct GetLatestBlockHeaderRequest {
        #[otopr(1)]
        pub is_sealed: bool,
    }

    access_api! {
        rpc Ping(PingRequest) returns (PingResponse);
        rpc GetLatestBlockHeader(GetLatestBlockHeaderRequest)
            returns (BlockHeaderResponse);
    }
    
}