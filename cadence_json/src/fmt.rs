use std::fmt;

use crate::{AddressOwned, AddressRef, CompositeOwned, ValueOwned};

impl fmt::Debug for CompositeOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();

        map.entry(&"id", &self.id);

        for field in &self.fields {
            map.entry(&field.name, &field.value);
        }

        map.finish()
    }
}

impl fmt::Debug for AddressOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for AddressOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = hex::encode(&self.data);
        assert_ne!(hex.len(), 0);
        f.write_str("0x")?;
        f.write_str(&hex)
    }
}

impl fmt::Debug for AddressRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for AddressRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = hex::encode(self.data);
        assert_ne!(hex.len(), 0);
        f.write_str("0x")?;
        f.write_str(&hex)
    }
}

impl fmt::Debug for ValueOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "Void"),
            Self::Int(arg0) => arg0.fmt(f),
            Self::Int8(arg0) => arg0.fmt(f),
            Self::Int16(arg0) => arg0.fmt(f),
            Self::Int32(arg0) => arg0.fmt(f),
            Self::Int64(arg0) => arg0.fmt(f),
            Self::Int128(arg0) => arg0.fmt(f),
            Self::Int256(arg0) => arg0.fmt(f),
            Self::UInt(arg0) => arg0.fmt(f),
            Self::UInt8(arg0) => arg0.fmt(f),
            Self::UInt16(arg0) => arg0.fmt(f),
            Self::UInt32(arg0) => arg0.fmt(f),
            Self::UInt64(arg0) => arg0.fmt(f),
            Self::UInt128(arg0) => arg0.fmt(f),
            Self::UInt256(arg0) => arg0.fmt(f),
            Self::Fix64(arg0) => arg0.fmt(f),
            Self::UFix64(arg0) => arg0.fmt(f),
            Self::Word8(arg0) => arg0.fmt(f),
            Self::Word16(arg0) => arg0.fmt(f),
            Self::Word32(arg0) => arg0.fmt(f),
            Self::Word64(arg0) => arg0.fmt(f),
            Self::Bool(arg0) => arg0.fmt(f),
            Self::Optional(arg0) => arg0.fmt(f),
            Self::String(arg0) => arg0.fmt(f),
            Self::Address(arg0) => arg0.fmt(f),
            Self::Array(arg0) => arg0.fmt(f),
            Self::Dictionary(arg0) => {
                let mut map = f.debug_map();
                for entry in arg0 {
                    map.entry(&entry.key, &entry.value);
                }
                map.finish()
            }
            Self::Struct(arg0)
            | Self::Resource(arg0)
            | Self::Event(arg0)
            | Self::Contract(arg0)
            | Self::Enum(arg0) => {
                let mut dbg = f.debug_struct(self.ty().as_str());
                dbg.field("id", &arg0.id);
                for field in &arg0.fields {
                    dbg.field(&field.name, &field.value);
                }
                dbg.finish()
            }
            Self::Path(arg0) => f.debug_tuple("Path").field(arg0).finish(),
            Self::Type(arg0) => arg0.fmt(f),
            Self::Capability(arg0) => arg0.fmt(f),
        }
    }
}
