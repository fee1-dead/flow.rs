use crate::transaction::rlp::{rlp_encode_transaction_envelope, rlp_encode_transaction_payload};

use super::fixtures::Test;

#[derive(serde::Deserialize, Debug)]
pub struct TxTest {
    #[serde(rename = "in")]
    pub tx_in: TxIn,
    #[serde(rename = "out")]
    pub tx_out: TxOut,
}

impl Test for TxTest {
    fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("{:?}", self);
        let TxTest {
            tx_in:
                TxIn {
                    cadence,
                    arguments,
                    ref_block,
                    compute_limit,
                    proposal_key:
                        ProposalKey {
                            address,
                            key_id,
                            sequence_number,
                        },
                    payer,
                    authorizers,
                    payload_signatures,
                },
            tx_out: TxOut { payload, envelope },
        } = self;
        let reference_block_id = hex::decode(ref_block).unwrap();
        let proposal_key_address = hex::decode(address).unwrap();
        let payer = hex::decode(payer).unwrap();
        let authorizers = authorizers.iter().map(hex::decode).map(Result::unwrap);
        let mut stream = rlp::RlpStream::new();

        rlp_encode_transaction_payload(
            &mut stream,
            &cadence,
            &arguments,
            &reference_block_id,
            compute_limit,
            &proposal_key_address,
            key_id,
            sequence_number,
            &payer,
            authorizers.clone(),
        );

        assert_eq!(payload, hex::encode(stream.as_raw()));

        stream.clear();

        rlp_encode_transaction_envelope(
            &mut stream,
            &cadence,
            arguments,
            reference_block_id,
            compute_limit,
            proposal_key_address,
            key_id,
            sequence_number,
            payer,
            authorizers,
            payload_signatures.into_iter().map(|ps| {
                (
                    hex::decode(ps.address).unwrap(),
                    ps.key_id,
                    hex::decode(ps.sig).unwrap(),
                )
            }),
        );

        assert_eq!(envelope, hex::encode(stream.as_raw()));

        Ok(())
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct TxIn {
    pub cadence: String,
    pub arguments: Vec<String>,
    #[serde(rename = "refBlock")]
    pub ref_block: String,
    #[serde(rename = "computeLimit")]
    pub compute_limit: u64,
    #[serde(rename = "proposalKey")]
    pub proposal_key: ProposalKey,
    pub payer: String,
    pub authorizers: Vec<String>,
    #[serde(rename = "payloadSigs")]
    pub payload_signatures: Vec<PayloadSignature>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TxOut {
    pub payload: String,
    pub envelope: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ProposalKey {
    pub address: String,
    #[serde(rename = "keyId")]
    pub key_id: u64,
    #[serde(rename = "sequenceNum")]
    pub sequence_number: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct PayloadSignature {
    pub address: String,
    #[serde(rename = "keyId")]
    pub key_id: u32,
    pub sig: String,
}

crate::test_fixtures!(TxTest, "tx-encoding.json", test_tx_encoding);
