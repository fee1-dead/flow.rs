use std::str::FromStr;

/// A type in the cadence type system that needs to delegate serde implementation to a newtype wrapper.
///
/// For example, integers are strings within the Cadence-JSON interchange format, so we need to use
/// the `FromStr` and `Display` implementations to (de)serialize instead.
///
/// # Safety
///
/// `Self::Wrapped` must have the same layout as `Self`, i.e. they are safely transmutable.
///
/// Make sure that `Self::Wrapped` is a newtype struct annotated with #[repr(transparent)]
pub unsafe trait Wrap {
    type Wrapped;
}

pub fn wrap<T: Wrap>(of: &T) -> &T::Wrapped {
    // Safety: the wrapper is guarranteed to be #[repr(transparent)] over `T`.
    unsafe { &*(of as *const T as *const T::Wrapped) }
}

macro_rules! wrapper {
    ($Name:ident($ty:path)) => {
        #[derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay)]
        #[repr(transparent)]
        pub struct $Name(pub $ty);

        impl FromStr for $Name {
            type Err = <$ty as FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map($Name)
            }
        }

        impl std::fmt::Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        unsafe impl Wrap for $ty {
            type Wrapped = $Name;
        }
    };
}

wrapper!(BigUint(num_bigint::BigUint));
wrapper!(BigInt(num_bigint::BigInt));
wrapper!(I8(i8));
wrapper!(I16(i16));
wrapper!(I32(i32));
wrapper!(I64(i64));
wrapper!(I128(i128));
wrapper!(U8(u8));
wrapper!(U16(u16));
wrapper!(U32(u32));
wrapper!(U64(u64));
wrapper!(U128(u128));

#[derive(serde::Deserialize)]
pub struct TypeDe {
    #[serde(rename = "staticType")]
    pub static_type: String
}

#[derive(serde::Serialize)]
pub struct TypeSer<'a> {
    #[serde(rename = "staticType")]
    pub static_type: &'a str,
}
