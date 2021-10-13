use serde::{ser::SerializeStruct, Serialize};

use crate::ValueRef;

use crate::wrapper::*;

impl<'a> Serialize for ValueRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let ValueRef::Void = self {
            let mut st = serializer.serialize_struct("S", 1)?;
            st.serialize_field("type", "Void")?;
            return st.end();
        }
        let mut st = serializer.serialize_struct("S", 2)?;
        macro_rules! match_helper {
            (match self {
                $(ValueRef::$Variant:ident($pat:pat) => $exp:expr),+,
                _ => unreachable!(),
            }) => {
                match self {
                    $(ValueRef::$Variant($pat) => {
                        st.serialize_field("type", stringify!($Variant))?;
                        st.serialize_field("value", $exp)?;
                        st.end()
                    })+
                    _ => unreachable!(),
                }
            }
        }
        match_helper! {
            match self {
                ValueRef::Int(v) => wrap(v),
                ValueRef::Int8(v) => wrap(v),
                ValueRef::Int16(v) => wrap(v),
                ValueRef::Int32(v) => wrap(v),
                ValueRef::Int64(v) => wrap(v),
                ValueRef::Int128(v) => wrap(v),
                ValueRef::Int256(v) => wrap(v),
                ValueRef::UInt(v) => wrap(v),
                ValueRef::UInt8(v) => wrap(v),
                ValueRef::UInt16(v) => wrap(v),
                ValueRef::UInt32(v) => wrap(v),
                ValueRef::UInt64(v) => wrap(v),
                ValueRef::UInt128(v) => wrap(v),
                ValueRef::UInt256(v) => wrap(v),
                ValueRef::Fix64(f) => f,
                ValueRef::UFix64(f) => f,
                ValueRef::Word8(v) => wrap(&v.0),
                ValueRef::Word16(v) => wrap(&v.0),
                ValueRef::Word32(v) => wrap(&v.0),
                ValueRef::Word64(v) => wrap(&v.0),
                ValueRef::Bool(b) => b,
                ValueRef::Optional(o) => o,
                ValueRef::String(s) => s,
                ValueRef::Address(a) => a,
                ValueRef::Array(v) => v,
                ValueRef::Dictionary(v) => v,
                ValueRef::Struct(v) => v,
                ValueRef::Resource(v) => v,
                ValueRef::Event(v) => v,
                ValueRef::Contract(v) => v,
                ValueRef::Enum(v) => v,
                ValueRef::Path(v) => v,
                ValueRef::Type(v) => v,
                ValueRef::Capability(v) => v,
                _ => unreachable!(),
            }
        }
    }
}
