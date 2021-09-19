use std::{error::Error, future::Future, pin::Pin};

use prost::Message;
use tonic::{Request, body::BoxBody, client::{Grpc, GrpcService}, codec::ProstCodec, codegen::{Body, http::uri::PathAndQuery}, transport::Channel};

/// A gRPC client.
///
/// This trait is very simple to implement.
/// We recommend implementing this for &mut T instead of just T. 
pub trait GrpcClient<I, O> {
    type Output;
    fn send(self, input: I) -> Self::Output;
}

pub struct FlowClient<T> {
    inner: T,
}

pub type TonicFlowClient<Service> = FlowClient<Grpc<Service>>;
pub type TonicHyperFlowClient = TonicFlowClient<Channel>;

impl<'a, I, O, Service> GrpcClient<I, O> for &'a mut Grpc<Service>
    where
        I: FlowRequest<O> + Send + 'static,
        O: Message + Send + Default + 'static,
        Service: GrpcService<BoxBody> + 'static,
        Service::ResponseBody: Body + Send + Sync + 'static,
        <Service::ResponseBody as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>>, 
{
    type Output = Pin<Box<dyn Future<Output = Result<tonic::Response<O>, Box<dyn Error + Send + Sync>>> + 'a>>;
    fn send(self, input: I) -> Pin<Box<dyn Future<Output = Result<tonic::Response<O>, Box<dyn Error + Send + Sync>>> + 'a>> {
        Box::pin(async move {
            self.ready().await.map_err(|error| error.into())?;
            Ok(
                self.unary(
                    Request::new(input), 
                    PathAndQuery::from_static(I::PATH), 
                    ProstCodec::default()
                ).await?
            )
        })
    }
}

use crate::{protobuf::access::*, requests::FlowRequest};

macro_rules! define_reqs {
    ($($(#[$meta:meta])* $vis:vis fn $fn_name:ident($($tt:tt)*) $input:ty => $output:ty { $expr:expr })+) => {
        $($(#[$meta])*
        $vis fn $fn_name<'a>(&'a mut self,$($tt)*) -> <&'a mut Inner as GrpcClient<$input, $output>>::Output
            where
                &'a mut Inner: GrpcClient<$input, $output>,
        {
            self.send($expr)
        })+
    }
}

impl<Inner> FlowClient<Inner> {
    pub fn send<'a, T, U>(&'a mut self, input: T) -> <&'a mut Inner as GrpcClient<T, U>>::Output
        where 
            &'a mut Inner: GrpcClient<T, U>,
    {
        self.inner.send(input)
    }

    define_reqs! {
        /// Shortcut for `self.send(PingRequest {})`.
        pub fn ping() PingRequest => PingResponse {
            PingRequest {}
        }

        /// Returns information of the latest block.
        pub fn latest_block_header(is_sealed: bool) GetLatestBlockHeaderRequest => BlockHeaderResponse {
            GetLatestBlockHeaderRequest { is_sealed }
        }
    }
}

impl TonicHyperFlowClient {
    pub fn mainnet() -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            inner: Grpc::new(
                Channel::from_static("http://access.mainnet.nodes.onflow.org:9000").connect_lazy()?,
            ),
        })
    }
}