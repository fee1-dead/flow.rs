use otopr::DecodableMessage;

#[derive(DecodableMessage, Default, Debug)]
pub struct Event {
    pub ty: String,
    pub transaction_id: Vec<u8>,
    pub transaction_index: u32,
    pub event_index: u32,
    pub payload: Vec<u8>,
}

impl Event {
    pub fn parse_payload(&self) -> serde_json::Result<cadence_json::ValueOwned> {
        serde_json::from_slice(&self.payload)
    }
}
