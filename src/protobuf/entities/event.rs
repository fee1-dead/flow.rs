use otopr::DecodableMessage;

#[derive(DecodableMessage, Default)]
pub struct Event {
    pub r#type: String,
    pub transaction_id: Vec<u8>,
    pub transaction_index: u32,
    pub event_index: u32,
    pub payload: Vec<u8>,
}
