use std::marker::PhantomData;

use otopr::decoding::{DecodableMessage, Deserializer};
use otopr::encoding::EncodableMessage;
use otopr::encoding::ProtobufSerializer;
use tonic::codec::{Codec, Decoder, Encoder};
use tonic::Status;

use bytes::BufMut;

/// A buffer that contains a preencoded message.
pub struct PreEncode(Box<[u8]>);

impl PreEncode {
    /// Creates an instance of `PreEncode` by encoding a message.
    pub fn new<T: EncodableMessage>(msg: &T) -> Self {
        let cap = msg.encoded_size();
        let mut buf = Vec::with_capacity(cap);
        msg.encode(&mut ProtobufSerializer::new(&mut buf));
        Self(buf.into_boxed_slice())
    }
}

pub struct OtoprCodec<U>(PhantomData<U>);

impl<U> Default for OtoprCodec<U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<U> Codec for OtoprCodec<U>
where
    U: for<'a> DecodableMessage<'a> + Default + Send + 'static,
{
    type Encode = PreEncode;

    type Decode = U;

    type Encoder = PEnc;

    type Decoder = PDec<U>;

    fn encoder(&mut self) -> Self::Encoder {
        PEnc
    }

    fn decoder(&mut self) -> Self::Decoder {
        PDec(PhantomData)
    }
}

pub struct PEnc;
pub struct PDec<T>(PhantomData<T>);

unsafe impl<T> Sync for PDec<T> {}

impl Encoder for PEnc {
    type Item = PreEncode;

    type Error = Status;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        dst.put_slice(&item.0);
        Ok(())
    }
}

impl<T: for<'de> DecodableMessage<'de> + Default> Decoder for PDec<T> {
    type Item = T;

    type Error = Status;

    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let mut des = Deserializer::new(src);
        match T::decode(&mut des) {
            Ok(t) => Ok(Some(t)),
            Err(e) => Err(match e {
                otopr::decoding::DecodingError::Eof => Status::resource_exhausted("reached eof"),
                otopr::decoding::DecodingError::VarIntOverflow => {
                    Status::invalid_argument("scalar overflow")
                }
                otopr::decoding::DecodingError::Utf8Error(e) => {
                    Status::invalid_argument(&format!("{}", e))
                }
                otopr::decoding::DecodingError::UnknownWireType(u) => {
                    Status::invalid_argument(&format!("unknown wire type: {}", u))
                }
            }),
        }
    }
}
