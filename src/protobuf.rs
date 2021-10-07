use otopr::DecodableMessage;
#[derive(Clone, Copy, Default, DecodableMessage, Debug, PartialEq, Eq)]
pub struct Timestamp {
    #[otopr(1)]
    pub seconds: i64,
    #[otopr(2)]
    pub nanos: i32,
}

mod entities;
pub use entities::*;

pub mod access;
