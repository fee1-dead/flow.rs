use std::{error::Error, future::Future, pin::Pin};

use otopr::decoding::DecodableMessage;
use tonic::{
    body::BoxBody,
    client::{Grpc, GrpcService},
    codegen::{http::uri::PathAndQuery, Body},
    transport::Channel,
    Request,
};

use crate::{
    codec::{OtoprCodec, PreEncode},
    RepSlice, TransactionE,
};

/// A gRPC client.
pub trait GrpcClient<I, O> {
    type Error: Into<Box<dyn Error + Send + Sync>>;
    fn send<'a>(
        &'a mut self,
        input: I,
    ) -> Pin<Box<dyn Future<Output = Result<O, Self::Error>> + 'a>>;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct FlowClient<T> {
    inner: T,
}

pub type TonicFlowClient<Service> = FlowClient<Grpc<Service>>;
pub type TonicHyperFlowClient = TonicFlowClient<Channel>;

impl<I, O, Service> GrpcClient<I, O> for Grpc<Service>
where
    I: FlowRequest<O> + Send + Sync,
    O: for<'b> DecodableMessage<'b> + Send + Sync + Default + 'static,
    Service: GrpcService<BoxBody> + 'static,
    Service::ResponseBody: Body + Send + Sync + 'static,
    <Service::ResponseBody as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Error = Box<dyn Error + Send + Sync>;
    fn send<'a>(
        &'a mut self,
        input: I,
    ) -> Pin<Box<dyn Future<Output = Result<O, Box<dyn Error + Send + Sync>>> + 'a>> {
        let preenc = PreEncode::new(&input);
        Box::pin(async move {
            self.ready().await.map_err(|error| error.into())?;
            Ok(self
                .unary(
                    Request::new(preenc),
                    PathAndQuery::from_static(I::PATH),
                    OtoprCodec::default(),
                )
                .await?
                .into_inner())
        })
    }
}

use crate::{protobuf::*, requests::FlowRequest};

macro_rules! define_reqs {
    ($($(#[$meta:meta])* $vis:vis fn $fn_name:ident$(<($($ttss:tt)*)>)?($($tt:tt)*) $input:ty => $output:ty $(where ($($tts:tt)*))? { $expr:expr })+) => {
        $($(#[$meta])*
        $vis fn $fn_name<'grpc, $($($ttss)*)?>(&'grpc mut self,$($tt)*) -> Pin<Box<dyn Future<Output = Result<$output, Inner::Error>> + 'grpc>>
            where
                Inner: GrpcClient<$input, $output>,
                $($($tts)*)?
        {
            self.send($expr)
        })+
    }
}

impl<Inner> FlowClient<Inner> {
    #[inline]
    pub const fn new(inner: Inner) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> Inner {
        self.inner
    }

    pub fn send<'a, T, U>(
        &'a mut self,
        input: T,
    ) -> Pin<Box<dyn Future<Output = Result<U, Inner::Error>> + 'a>>
    where
        Inner: GrpcClient<T, U>,
    {
        self.inner.send(input)
    }

    define_reqs! {
        /// Shortcut for `self.send(PingRequest {})`.
        pub fn ping() PingRequest => PingResponse {
            PingRequest {}
        }
        pub fn latest_block_header(is_sealed: bool) GetLatestBlockHeaderRequest => BlockHeaderResponse {
            GetLatestBlockHeaderRequest { is_sealed }
        }
        pub fn block_header_by_height(height: u64) GetBlockHeaderByHeightRequest => BlockHeaderResponse {
            GetBlockHeaderByHeightRequest { height }
        }
        pub fn block_header_by_id<('a)>(id: &'a [u8]) GetBlockHeaderByIdRequest<'a> => BlockHeaderResponse {
            GetBlockHeaderByIdRequest { id }
        }
        pub fn latest_block(is_sealed: bool) GetLatestBlockRequest => BlockResponse {
            GetLatestBlockRequest { is_sealed }
        }
        pub fn block_by_height(height: u64) GetBlockByHeightRequest => BlockResponse {
            GetBlockByHeightRequest { height }
        }
        pub fn block_by_id<('a)>(id: &'a [u8]) GetBlockByIdRequest<'a> => BlockResponse {
            GetBlockByIdRequest { id }
        }
        pub fn collection_by_id<('a)>(id: &'a [u8]) GetCollectionByIdRequest<'a> => CollectionResponse {
            GetCollectionByIdRequest { id }
        }
        pub fn events_for_height_range<('a)>(r#type: &'a str, start_height: u64, end_height: u64) GetEventsForHeightRangeRequest<'a> => EventsResponse {
            GetEventsForHeightRangeRequest { r#type, start_height, end_height }
        }
        pub fn execute_script_at_latest_block<('a)>(script: &'a [u8], arguments: &'a [&'a [u8]]) ExecuteScriptAtLatestBlockRequest<'a> => ExecuteScriptResponse {
            ExecuteScriptAtLatestBlockRequest { script, arguments: RepSlice::new(arguments.into()) }
        }
        pub fn account_at_latest_block<('a)>(address: &'a [u8]) GetAccountAtLatestBlockRequest<'a> => AccountResponse {
            GetAccountAtLatestBlockRequest { id: address }
        }
        pub fn send_transaction<(
            Script,
            Arguments,
            ReferenceBlockId,
            ProposalKeyAddress,
            Payer,
            Authorizers,
            PayloadSignatures,
            EnvelopeSignatures,
        )>(transaction: TransactionE<
            Script,
            Arguments,
            ReferenceBlockId,
            ProposalKeyAddress,
            Payer,
            Authorizers,
            PayloadSignatures,
            EnvelopeSignatures,
        >) SendTransactionRequest<
            Script,
            Arguments,
            ReferenceBlockId,
            ProposalKeyAddress,
            Payer,
            Authorizers,
            PayloadSignatures,
            EnvelopeSignatures,
        > => SendTransactionResponse
        {
            SendTransactionRequest { transaction }
        }
    }
}

impl TonicHyperFlowClient {
    pub fn mainnet() -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            inner: Grpc::new(
                Channel::from_static("http://access.mainnet.nodes.onflow.org:9000")
                    .connect_lazy()?,
            ),
        })
    }

    pub fn testnet() -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            inner: Grpc::new(
                Channel::from_static("http://access.devnet.nodes.onflow.org:9000")
                    .connect_lazy()?,
            ),
        })
    }
}
