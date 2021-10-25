use otopr::{encoding::Encodable, DecodableMessage};

#[derive(Clone, Copy, Default, DecodableMessage, Debug, PartialEq, Eq, Hash)]
pub struct Timestamp {
    #[otopr(1)]
    pub seconds: i64,
    #[otopr(2)]
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Seal {
    Sealed,
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
