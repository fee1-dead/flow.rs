use std::marker::PhantomData;

use otopr::encoding::ProtobufSerializer;
use otopr::decoding::{DecodableMessage, Deserializer};
use otopr::traits::EncodableMessage;
use tonic::Status;
use tonic::codec::{Codec, Decoder, Encoder};

pub struct OtoprCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Default for OtoprCodec<T, U> {
    fn default() -> Self {
        Self(PhantomData)
    }
} 

impl<T, U> Codec for OtoprCodec<T, U>
where
    T: EncodableMessage + Send + 'static,
    U: for<'a> DecodableMessage<'a> + Default + Send + 'static,
{
    type Encode = T;

    type Decode = U;

    type Encoder = PEnc<T>;

    type Decoder = PDec<U>;

    fn encoder(&mut self) -> Self::Encoder {
        PEnc(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        PDec(PhantomData)
    }
}

pub struct PEnc<T>(PhantomData<T>);
pub struct PDec<T>(PhantomData<T>);

unsafe impl<T> Sync for PEnc<T> {}
unsafe impl<T> Sync for PDec<T> {}

impl<T: EncodableMessage> Encoder for PEnc<T> {
    type Item = T;

    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        let mut se = ProtobufSerializer::new(dst);
        item.encode(&mut se);
        Ok(())
    }
}

impl<T: for<'de> DecodableMessage<'de> + Default> Decoder for PDec<T> {
    type Item = T;

    type Error = Status;

    fn decode(&mut self, src: &mut tonic::codec::DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let mut des = Deserializer::new(src);
        match T::decode(&mut des) {
            Ok(t) => Ok(Some(t)),
            Err(e) => Err(match e {
                otopr::decoding::DecodingError::Eof => Status::resource_exhausted("reached eof"),
                otopr::decoding::DecodingError::VarIntOverflow => Status::invalid_argument("scalar overflow"),
                otopr::decoding::DecodingError::Utf8Error(e) => Status::invalid_argument(&format!("{}", e)),
                otopr::decoding::DecodingError::UnknownWireType(u) => Status::invalid_argument(&format!("unknown wire type: {}", u)),
            }),
        }
    }
}