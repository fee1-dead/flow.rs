//! Protobuf objects to decode/encode.

use otopr::{encoding::Encodable, DecodableMessage};

/// A protobuf timestamp.
#[derive(Clone, Copy, Default, DecodableMessage, Debug, PartialEq, Eq, Hash)]
pub struct Timestamp {
    /// The seconds component of the timestamp.
    pub seconds: i64,
    /// The nanoseconds component of the timestamp.
    pub nanos: i32,
}

/// Seal: the status of an entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Seal {
    /// Sealed: An entity was successful and permanently recorded on the blockchain.
    Sealed,
    /// The entity is not yet sealed.
    NotSealed,
}

impl Encodable for Seal {
    type Wire = <bool as Encodable>::Wire;

    fn encoded_size<V: otopr::VarInt>(&self, field_number: V) -> usize {
        field_number.size() + 1
    }

    fn encode(&self, s: &mut otopr::encoding::ProtobufSerializer<impl bytes::BufMut>) {
        let is_sealed = matches!(self, Seal::Sealed);
        is_sealed.encode(s)
    }
}
