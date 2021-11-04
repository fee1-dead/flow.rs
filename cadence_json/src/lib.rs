use std::borrow::Cow;
use std::fmt::Display;
use std::num::Wrapping;
use std::str::FromStr;

use hex::FromHexError;
pub use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use serde_with::*;

mod fmt;

mod fixed;
pub use fixed::*;

pub mod de;
pub mod ser;

pub(crate) mod wrapper;

#[cfg(test)]
mod tests;

#[derive(SerializeDisplay, DeserializeFromStr, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PathDomain {
    Storage,
    Private,
    Public,
}

impl FromStr for PathDomain {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "storage" => Self::Storage,
            "private" => Self::Private,
            "public" => Self::Public,
            _ => return Err("invalid path domain"),
        })
    }
}

impl Display for PathDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Storage => "storage",
            Self::Private => "private",
            Self::Public => "public",
        })
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathRef<'a> {
    pub domain: PathDomain,
    pub identifier: &'a str,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PathOwned {
    pub domain: PathDomain,
    pub identifier: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct CompositeFieldRef<'a> {
    pub name: &'a str,
    pub value: ValueRef<'a>,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositeRef<'a> {
    pub id: &'a str,
    pub fields: &'a [CompositeFieldRef<'a>],
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompositeFieldOwned {
    pub name: String,
    pub value: ValueOwned,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CompositeOwned {
    pub id: String,
    pub fields: Vec<CompositeFieldOwned>,
}

impl CompositeOwned {
    pub fn find_field<'a>(&'a self, name: &str) -> Option<&'a ValueOwned> {
        self.fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| &f.value)
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct EntryRef<'a> {
    pub key: ValueRef<'a>,
    pub value: ValueRef<'a>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EntryOwned {
    pub key: ValueOwned,
    pub value: ValueOwned,
}

#[derive(DeserializeFromStr, SerializeDisplay, Clone, PartialEq, Eq)]
pub struct AddressOwned {
    pub data: Box<[u8]>,
}

impl FromStr for AddressOwned {
    type Err = Cow<'static, str>;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s
            .strip_prefix("0x")
            .ok_or(Cow::Borrowed("Address does not start with 0x"))?;
        hex::decode(s)
            .map(|data| AddressOwned {
                data: data.into_boxed_slice(),
            })
            .map_err(|e| match e {
                FromHexError::OddLength => Cow::Borrowed("Odd number of digits"),
                FromHexError::InvalidStringLength => Cow::Borrowed("Invalid string length"),
                e => e.to_string().into(),
            })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct AddressRef<'a> {
    pub data: &'a [u8],
}

impl Serialize for AddressRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityOwned {
    pub path: String,
    pub address: AddressOwned,
    #[serde(rename = "borrowType")]
    pub borrow_type: String,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityRef<'a> {
    pub path: &'a str,
    pub address: AddressRef<'a>,
    pub borrow_type: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueRef<'a> {
    Void,
    Int(BigInt),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Int256(BigInt),
    UInt(BigUint),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    UInt256(BigUint),
    Fix64(Fix64),
    UFix64(UFix64),
    Word8(Wrapping<u8>),
    Word16(Wrapping<u16>),
    Word32(Wrapping<u32>),
    Word64(Wrapping<u64>),
    Bool(bool),
    Optional(Option<Box<ValueRef<'a>>>),
    String(&'a str),
    Address(AddressRef<'a>),
    Array(&'a [ValueRef<'a>]),
    Dictionary(&'a [EntryRef<'a>]),
    Struct(CompositeRef<'a>),
    Resource(CompositeRef<'a>),
    Event(CompositeRef<'a>),
    Contract(CompositeRef<'a>),
    Enum(CompositeRef<'a>),
    Path(PathRef<'a>),
    Type(&'a str),
    Capability(CapabilityRef<'a>),
}

/// An owned Cadence value.
#[derive(Clone, PartialEq, Eq)]
pub enum ValueOwned {
    Void,
    Int(BigInt),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Int256(BigInt),
    UInt(BigUint),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    UInt256(BigUint),
    Fix64(Fix64),
    UFix64(UFix64),
    Word8(Wrapping<u8>),
    Word16(Wrapping<u16>),
    Word32(Wrapping<u32>),
    Word64(Wrapping<u64>),
    Bool(bool),
    Optional(Option<Box<ValueOwned>>),
    String(String),
    Address(AddressOwned),
    Array(Vec<ValueOwned>),
    Dictionary(Vec<EntryOwned>),
    Struct(CompositeOwned),
    Resource(CompositeOwned),
    Event(CompositeOwned),
    Contract(CompositeOwned),
    Enum(CompositeOwned),
    Path(PathOwned),
    Type(String),
    Capability(CapabilityOwned),
}

impl ValueOwned {
    pub fn expect_address(&self) -> &AddressOwned {
        match self {
            Self::Address(addr) => addr,
            _ => panic!("Expected Address, found {}", self.ty()),
        }
    }
    pub fn as_address(&self) -> Option<&AddressOwned> {
        match self {
            Self::Address(addr) => Some(addr),
            _ => None,
        }
    }
}

macro_rules! ty {
    (pub enum Type {
        Void,
        $($Variant:ident),*$(,)?
    }) => {
        #[derive(SerializeDisplay, DeserializeFromStr)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Type {
            Void,
            $($Variant),*
        }
        impl Type {
            pub const fn as_str(self) -> &'static str {
                match self {
                    Self::Void => "Void",
                    $(Type::$Variant => stringify!($Variant),)*
                }
            }
        }
        impl std::str::FromStr for Type {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Type, &'static str> {
                match s {
                    "Void" => Ok(Self::Void),
                    $(stringify!($Variant) => Ok(Type::$Variant),)*
                    _ => Err("invalid type string"),
                }
            }
        }
        impl ValueRef<'_> {
            pub fn ty(&self) -> Type {
                match self {
                    Self::Void => Type::Void,
                    $(Self::$Variant(_) => Type::$Variant,)*
                }
            }
        }
        impl ValueOwned {
            pub fn ty(&self) -> Type {
                match self {
                    Self::Void => Type::Void,
                    $(Self::$Variant(_) => Type::$Variant,)*
                }
            }
        }
    };
}
ty! {
    pub enum Type {
        Void,
        Optional,
        Bool,
        String,
        Address,
        UInt,
        UInt8,
        UInt16,
        UInt32,
        UInt64,
        UInt128,
        UInt256,
        Int,
        Int8,
        Int16,
        Int32,
        Int64,
        Int128,
        Int256,
        Word8,
        Word16,
        Word32,
        Word64,
        UFix64,
        Fix64,
        Array,
        Dictionary,
        Struct,
        Resource,
        Event,
        Contract,
        Enum,
        Path,
        Type,
        Capability,
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
