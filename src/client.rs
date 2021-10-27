//! ## Flow gRPC connections
//!
//!

use std::{error::Error, future::Future, pin::Pin};

use otopr::{decoding::DecodableMessage, encoding::EncodableMessage};
use tonic::{
    body::BoxBody,
    client::{Grpc, GrpcService},
    transport::Channel,
    Request,
};

use http::uri::PathAndQuery;
use http_body::Body;

use crate::access::*;
use crate::transaction::TransactionE;
use crate::{
    codec::{OtoprCodec, PreEncode},
    entities::{Account, Block, BlockHeader, Collection},
    transaction::TransactionD,
};

/// A gRPC client trait.
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

pub type GrpcSendResult<'a, Output> =
    Pin<Box<dyn Future<Output = Result<Output, Box<dyn Error + Send + Sync>>> + 'a>>;

impl<I, O, Service> GrpcClient<I, O> for Grpc<Service>
where
    I: FlowRequest<O> + Send + Sync + EncodableMessage,
    O: for<'b> DecodableMessage<'b> + Send + Sync + Default + 'static,
    Service: GrpcService<BoxBody> + 'static,
    Service::ResponseBody: Body + Send + Sync + 'static,
    <Service::ResponseBody as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Error = Box<dyn Error + Send + Sync>;
    fn send(&mut self, input: I) -> GrpcSendResult<O> {
        let preenc = PreEncode::new(&input);
        Box::pin(async move {
            self.ready().await.map_err(Into::into)?;
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

// Simple requests that constructs a request from parameters.
macro_rules! define_requests {
    ($($(#[$meta:meta])* $vis:vis async fn $fn_name:ident$(<($($ttss:tt)*)>)?($($tt:tt)*) $input:ty => $output:ty $(where ($($tts:tt)*))? { $expr:expr })+) => {
        $($(#[$meta])*
        $vis fn $fn_name<'grpc, $($($ttss)*)?>(&'grpc mut self,$($tt)*) -> Pin<Box<dyn Future<Output = Result<$output, Inner::Error>> + 'grpc>>
            where
                Inner: GrpcClient< $input, $output >,
                $($($tts)*)?
        {
            self.send($expr)
        })+
    }
}

// Requests that `.map()`s the futures before returning.
macro_rules! remapping_requests {
    ($($(#[$meta:meta])* $vis:vis async fn $fn_name:ident$(<($($ttss:tt)*)>)?($($tt:tt)*)
        $input:ty => $output:ty $(where ($($tts:tt)*))? {
            $expr:expr;
            remap = |$paramName:ident| -> $remappedty:ty $remap:block
        })+) => {
        $($(#[$meta])*
        $vis fn $fn_name<'grpc, $($($ttss)*)?>(&'grpc mut self,$($tt)*) ->
            futures_util::future::Map<
                Pin<Box<dyn Future<Output = Result<$output, Inner::Error>> + 'grpc>>,
                fn(Result<$output, Inner::Error>) -> Result<$remappedty, Inner::Error>,
            >
            where
                Inner: GrpcClient< $input, $output >,
                $($($tts)*)?
        {
            fn remap_ok($paramName: $output) -> $remappedty {
                $remap
            }
            fn remap<E>(res: Result<$output, E>) -> Result<$remappedty, E> {
                res.map(remap_ok)
            }
            use futures_util::FutureExt;
            self.send($expr).map(remap::<Inner::Error>)
        })+
    }
}

impl<Inner> FlowClient<Inner> {
    /// Wraps the inner client to gain access to helper functions to send requests.
    #[inline]
    pub const fn new(inner: Inner) -> Self {
        Self { inner }
    }

    /// Retrieve the inner client from this instance.
    #[inline]
    pub fn into_inner(self) -> Inner {
        self.inner
    }

    #[inline]
    pub fn as_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    /// Sends a request over the client.
    #[inline]
    pub fn send<'a, T, U>(
        &'a mut self,
        input: T,
    ) -> Pin<Box<dyn Future<Output = Result<U, Inner::Error>> + 'a>>
    where
        Inner: GrpcClient<T, U>,
    {
        self.inner.send(input)
    }

    define_requests! {
        /// Shortcut for `self.send(PingRequest {})`.
        pub async fn ping() PingRequest => PingResponse {
            PingRequest {}
        }
        pub async fn events_for_height_range<('a)>(r#type: &'a str, start_height: u64, end_height: u64) GetEventsForHeightRangeRequest<'a> => EventsResponse {
            GetEventsForHeightRangeRequest { r#type, start_height, end_height }
        }
        pub async fn execute_script_at_latest_block<(Script, Arguments)>(script: Script, arguments: Arguments) ExecuteScriptAtLatestBlockRequest<Script, Arguments> => ExecuteScriptResponse {
            ExecuteScriptAtLatestBlockRequest { script, arguments }
        }
        pub async fn send_transaction<(
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

    remapping_requests! {
        pub async fn transaction_by_id<('a)>(id: &'a [u8]) GetTransactionRequest<'a> => TransactionResponse {
            GetTransactionRequest { id };
            remap = |txn_response| -> TransactionD {
                txn_response.transaction
            }
        }
        pub async fn account_at_latest_block<('a)>(address: &'a [u8]) GetAccountAtLatestBlockRequest<'a> => AccountResponse {
            GetAccountAtLatestBlockRequest { id: address };
            remap = |acc_response| -> Account {
                acc_response.account
            }
        }
        pub async fn account_at_block_height<('a)>(address: &'a [u8], block_height: u64) GetAccountAtBlockHeightRequest<'a> => AccountResponse {
            GetAccountAtBlockHeightRequest { id: address, block_height };
            remap = |acc_response| -> Account {
                acc_response.account
            }
        }
        pub async fn latest_block_header(seal: Seal) GetLatestBlockHeaderRequest => BlockHeaderResponse {
            GetLatestBlockHeaderRequest { seal };
            remap = |header_response| -> BlockHeader {
                header_response.0
            }
        }
        pub async fn block_header_by_height(height: u64) GetBlockHeaderByHeightRequest => BlockHeaderResponse {
            GetBlockHeaderByHeightRequest { height };
            remap = |header_response| -> BlockHeader {
                header_response.0
            }
        }
        pub async fn block_header_by_id<('a)>(id: &'a [u8]) GetBlockHeaderByIdRequest<'a> => BlockHeaderResponse {
            GetBlockHeaderByIdRequest { id };
            remap = |header_response| -> BlockHeader {
                header_response.0
            }
        }
        pub async fn latest_block(seal: Seal) GetLatestBlockRequest => BlockResponse {
            GetLatestBlockRequest { seal };
            remap = |block_response| -> Block {
                block_response.0
            }
        }
        pub async fn block_by_height(height: u64) GetBlockByHeightRequest => BlockResponse {
            GetBlockByHeightRequest { height };
            remap = |block_response| -> Block {
                block_response.0
            }
        }
        pub async fn block_by_id<('a)>(id: &'a [u8]) GetBlockByIdRequest<'a> => BlockResponse {
            GetBlockByIdRequest { id };
            remap = |block_response| -> Block {
                block_response.0
            }
        }
        pub async fn collection_by_id<('a)>(id: &'a [u8]) GetCollectionByIdRequest<'a> => CollectionResponse {
            GetCollectionByIdRequest { id };
            remap = |collection_response| -> Collection {
                collection_response.collection
            }
        }
    }
}

impl TonicHyperFlowClient {
    /// Connects to a static endpoint. Does not connect until we try to send a request.
    pub fn connect_static(endpoint: &'static str) -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            inner: Grpc::new(Channel::from_static(endpoint).connect_lazy()?),
        })
    }

    pub fn connect_shared(
        endpoint: impl Into<bytes::Bytes>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            inner: Grpc::new(Channel::from_shared(endpoint)?.connect_lazy()?),
        })
    }

    /// Connects to the Mainnet access node provided by Dapper Labs.
    pub fn mainnet() -> Result<Self, tonic::transport::Error> {
        Self::connect_static("http://access.mainnet.nodes.onflow.org:9000")
    }

    /// Connects to the Testnet access node provided by Dapper Labs.
    pub fn testnet() -> Result<Self, tonic::transport::Error> {
        Self::connect_static("http://access.devnet.nodes.onflow.org:9000")
    }
}

impl<Inner, I, O> GrpcClient<I, O> for FlowClient<Inner>
where
    Inner: GrpcClient<I, O>,
{
    type Error = Inner::Error;

    #[inline]
    fn send<'a>(
        &'a mut self,
        input: I,
    ) -> Pin<Box<dyn Future<Output = Result<O, Self::Error>> + 'a>> {
        self.inner.send(input)
    }
}
