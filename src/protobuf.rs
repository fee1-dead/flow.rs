
pub mod access {
    use otopr::{EncodableMessage, DecodableMessage};

    use crate::requests::FlowRequest;

    #[derive(EncodableMessage)]
    pub struct PingRequest;

    impl FlowRequest<PingResponse> for PingRequest {
        const PATH: &'static str = "/flow.access.AccessAPI/Ping";
    }

    impl crate::requests::private::Sealed for PingRequest {}

    #[derive(DecodableMessage, Default)]
    pub struct PingResponse;
}