use std::{collections::HashMap, fmt, ops::Deref};

use cadence_json::AddressRef;

use crate::*;

struct Hexes<'a>(&'a Vec<Vec<u8>>);

impl fmt::Debug for Hexes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(Deref::deref).map(Hex))
            .finish()
    }
}

struct Hex<'a>(&'a [u8]);

impl fmt::Debug for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

struct Addrs<'a>(&'a [Vec<u8>]);

impl fmt::Debug for Addrs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(
                self.0
                    .iter()
                    .map(Deref::deref)
                    .map(|data| AddressRef { data }),
            )
            .finish()
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("id", &Hex(&self.id))
            .field("parent_id", &Hex(&self.parent_id))
            .field("height", &self.height)
            .field("timestamp", &self.timestamp)
            .field("collection_guarantees", &self.collection_guarantees)
            .field("block_seals", &self.block_seals)
            .field("signatures", &Hexes(&self.signatures))
            .finish()
    }
}

impl fmt::Debug for BlockSeal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockSeal")
            .field("block_id", &Hex(&self.block_id))
            .field("execution_receipt_id", &Hex(&self.execution_receipt_id))
            .field(
                "execution_receipt_signatures",
                &Hexes(&self.execution_receipt_signatures),
            )
            .field(
                "result_approval_signatures",
                &Hexes(&self.result_approval_signatures),
            )
            .finish()
    }
}

impl fmt::Debug for ProposalKeyD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProposalKey")
            .field(
                "address",
                &AddressRef {
                    data: &self.address,
                },
            )
            .field("key_id", &self.key_id)
            .field("sequence_number", &self.sequence_number)
            .finish()
    }
}

impl fmt::Debug for SignatureD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signature")
            .field(
                "address",
                &AddressRef {
                    data: &self.address,
                },
            )
            .field("key_id", &self.key_id)
            .field("signature", &Hex(&self.signature))
            .finish()
    }
}

impl fmt::Debug for TransactionD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Arguments<'a>(&'a [Vec<u8>]);
        impl fmt::Debug for Arguments<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list()
                    .entries(self.0.iter().map(|v| String::from_utf8_lossy(v)))
                    .finish()
            }
        }

        f.debug_struct("Transaction")
            .field("script", &String::from_utf8_lossy(&self.script))
            .field("arguments", &Arguments(&self.arguments))
            .field("reference_block_id", &Hex(&self.reference_block_id))
            .field("gas_limit", &self.gas_limit)
            .field("proposal_key", &self.proposal_key)
            .field("payer", &AddressRef { data: &self.payer })
            .field("authorizers", &Addrs(&self.authorizers))
            .field("payload_signatures", &self.payload_signatures)
            .field("envelope_signatures", &self.envelope_signatures)
            .finish()
    }
}

impl fmt::Debug for AccountKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountKey")
            .field("index", &self.index)
            .field("public_key", &Hex(&self.public_key))
            .field("sign_algo", &self.sign_algo)
            .field("hash_algo", &self.hash_algo)
            .field("weight", &self.weight)
            .field("sequence_number", &self.sequence_number)
            .field("revoked", &self.revoked)
            .finish()
    }
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Contracts<'a>(&'a HashMap<String, Vec<u8>>);

        impl fmt::Debug for Contracts<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_map()
                    .entries(
                        self.0
                            .iter()
                            .map(|(key, value)| (key, String::from_utf8_lossy(value))),
                    )
                    .finish()
            }
        }

        f.debug_struct("Account")
            .field(
                "address",
                &AddressRef {
                    data: &self.address,
                },
            )
            .field("balance", &self.balance)
            .field("code", &self.code)
            .field("keys", &self.keys)
            .field("contracts", &Contracts(&self.contracts))
            .finish()
    }
}

impl fmt::Debug for Collection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Collection")
            .field("id", &Hex(&self.id))
            .field("transactions", &Hexes(&self.transactions))
            .finish()
    }
}
