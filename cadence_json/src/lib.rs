use std::num::Wrapping;

pub use num_bigint::{BigUint, BigInt};

mod fixed;
pub use fixed::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PathDomain {
    Storage,
    Private,
    Public,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositeRef<'a> {
    pub id: &'a str,
    pub fields: &'a [(&'a str, ValueRef<'a>)],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositeOwned {
    pub id: String,
    pub fields: Vec<(String, ValueOwned)>,
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
    Address(&'a [u8]),
    Array(&'a [ValueRef<'a>]),
    Dictionary(&'a [ (ValueRef<'a>, ValueRef<'a>) ]),
    Struct(CompositeRef<'a>),
    Resource(CompositeRef<'a>),
    Event(CompositeRef<'a>),
    Contract(CompositeRef<'a>),
    Enum(CompositeRef<'a>),
    Path {
        domain: PathDomain,
        identifier: &'a str,
    },
    Type(&'a str),
    Capability {
        path: &'a str,
        address: &'a [u8],
        borrow_type: &'a str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    Address(Vec<u8>),
    Array(Vec<ValueOwned>),
    Dictionary(Vec<(ValueOwned, ValueOwned)>),
    Struct(CompositeOwned),
    Resource(CompositeOwned),
    Event(CompositeOwned),
    Contract(CompositeOwned),
    Enum(CompositeOwned),
    Path {
        domain: PathDomain,
        identifier: String,
    },
    Type(String),
    Capability {
        path: String,
        address: Vec<u8>,
        borrow_type: String,
    },
}