use std::num::Wrapping;

use serde::{
    de::{DeserializeSeed, Error, Visitor},
    Deserializer,
};

use super::wrapper::*;
use crate::ValueOwned;

struct ExpectedStr(&'static str);

impl<'de> Visitor<'de> for ExpectedStr {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "A string \"{}\"", self.0)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v != self.0 {
            Err(E::custom("invalid string"))
        } else {
            Ok(())
        }
    }
}

impl<'de> DeserializeSeed<'de> for ExpectedStr {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

struct StrMap<F: FnOnce(&str) -> Result<R, E>, R, E>(F);

impl<'de, F: FnOnce(&str) -> Result<R, E>, R, E: Error> DeserializeSeed<'de> for StrMap<F, R, E> {
    type Value = R;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de, F: FnOnce(&str) -> Result<R, E>, R, E: Error> Visitor<'de> for StrMap<F, R, E> {
    type Value = R;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a cadence type string")
    }

    fn visit_str<E1>(self, v: &str) -> Result<Self::Value, E1>
    where
        E1: Error,
    {
        self.0(v).map_err(E1::custom)
    }
}

struct CadenceObjectVisitor;

impl<'de> Visitor<'de> for CadenceObjectVisitor {
    type Value = ValueOwned;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "A Cadence-JSON Object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        use crate::Type::*;
        map.next_key_seed(ExpectedStr("type"))?
            .ok_or_else(|| A::Error::custom("expected type entry"))?;
        let ty: super::Type = map.next_value()?;
        if ty == super::Type::Void {
            return Ok(ValueOwned::Void);
        }
        map.next_key_seed(ExpectedStr("value"))?
            .ok_or_else(|| A::Error::custom("expected value entry"))?;
        Ok(match ty {
            Void => ValueOwned::Void,
            Optional => ValueOwned::Optional(map.next_value()?),
            Bool => ValueOwned::Bool(map.next_value()?),
            String => ValueOwned::String(map.next_value()?),
            Address => ValueOwned::Address(map.next_value()?),
            UInt => ValueOwned::UInt(map.next_value::<BigUint>()?.0),
            UInt8 => ValueOwned::UInt8(map.next_value::<U8>()?.0),
            UInt16 => ValueOwned::UInt16(map.next_value::<U16>()?.0),
            UInt32 => ValueOwned::UInt32(map.next_value::<U32>()?.0),
            UInt64 => ValueOwned::UInt64(map.next_value::<U64>()?.0),
            UInt128 => ValueOwned::UInt128(map.next_value::<U128>()?.0),
            UInt256 => ValueOwned::UInt256(map.next_value::<BigUint>()?.0),
            Int => ValueOwned::Int(map.next_value::<BigInt>()?.0),
            Int8 => ValueOwned::Int8(map.next_value::<I8>()?.0),
            Int16 => ValueOwned::Int16(map.next_value::<I16>()?.0),
            Int32 => ValueOwned::Int32(map.next_value::<I32>()?.0),
            Int64 => ValueOwned::Int64(map.next_value::<I64>()?.0),
            Int128 => ValueOwned::Int128(map.next_value::<I128>()?.0),
            Int256 => ValueOwned::Int256(map.next_value::<BigInt>()?.0),
            Word8 => ValueOwned::Word8(map.next_value::<U8>().map(|n| n.0).map(Wrapping)?),
            Word16 => ValueOwned::Word16(map.next_value::<U16>().map(|n| n.0).map(Wrapping)?),
            Word32 => ValueOwned::Word32(map.next_value::<U32>().map(|n| n.0).map(Wrapping)?),
            Word64 => ValueOwned::Word64(map.next_value::<U64>().map(|n| n.0).map(Wrapping)?),
            UFix64 => ValueOwned::UFix64(map.next_value()?),
            Fix64 => ValueOwned::Fix64(map.next_value()?),
            Array => ValueOwned::Array(map.next_value()?),
            Dictionary => ValueOwned::Dictionary(map.next_value()?),
            Struct => ValueOwned::Struct(map.next_value()?),
            Resource => ValueOwned::Resource(map.next_value()?),
            Event => ValueOwned::Event(map.next_value()?),
            Contract => ValueOwned::Contract(map.next_value()?),
            Enum => ValueOwned::Enum(map.next_value()?),
            Path => ValueOwned::Path(map.next_value()?),
            Type => ValueOwned::Type(map.next_value::<TypeDe>()?.static_type),
            Capability => ValueOwned::Capability(map.next_value()?),
        })
    }
}

impl<'de> serde::Deserialize<'de> for super::ValueOwned {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CadenceObjectVisitor)
    }
}
