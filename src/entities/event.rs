use cadence_json::ValueOwned;
use otopr::DecodableMessage;

/// An event is emitted as the result of a transaction execution.
///
/// Events are either user-defined events originating from a Cadence smart contract,
/// or built-in Flow system events.
#[derive(DecodableMessage, Default)]
pub struct Event {
    /// Fully-qualified unique type identifier for the event
    pub ty: String,
    /// ID of the transaction the event was emitted from
    pub transaction_id: Box<[u8]>,
    /// Zero-based index of the transaction within the block
    pub transaction_index: u32,
    /// Zero-based index of the event within the transaction
    pub event_index: u32,
    /// Event fields encoded as JSON-Cadence values
    pub payload: Box<[u8]>,
}

impl Event {
    /// Parses the payload of this event as a cadence JSON value.
    pub fn parse_payload_as_value(&self) -> serde_json::Result<cadence_json::ValueOwned> {
        serde_json::from_slice(&self.payload)
    }

    /// Parses the payload of this event.
    pub fn parse_payload(&self) -> serde_json::Result<cadence_json::CompositeOwned> {
        match self.parse_payload_as_value()? {
            ValueOwned::Event(composite) => Ok(composite),
            _ => panic!("Invalid payload for Event"),
        }
    }
}
