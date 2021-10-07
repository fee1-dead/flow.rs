use otopr::DecodableMessage;
use otopr::Message;

#[derive(DecodableMessage, Default, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    #[otopr(1)]
    pub id: Vec<u8>,
    #[otopr(2)]
    pub parent_id: Vec<u8>,
    #[otopr(3)]
    pub height: u64,
    #[otopr(4)]
    pub timestamp: Message<super::Timestamp>,
}
