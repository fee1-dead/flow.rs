//! Debug / Display implementations for structures.

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

use cadence_json::AddressRef;
use otopr::HasItem;

use crate::access::TransactionResultResponse;
use crate::entities::*;
use crate::transaction::*;

struct Hexes<H>(H);

impl<H> fmt::Debug for Hexes<H>
where
    H: IntoIterator + Copy,
    H::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.into_iter().map(Hex)).finish()
    }
}

struct Hex<H>(H);

impl<H: AsRef<[u8]>> fmt::Debug for Hex<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0.as_ref() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

struct Addrs<'a>(&'a [Box<[u8]>]);

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
            .field("signatures", &Hexes(&*self.signatures))
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
                &Hexes(&*self.execution_receipt_signatures),
            )
            .field(
                "result_approval_signatures",
                &Hexes(&*self.result_approval_signatures),
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
        struct Arguments<'a>(&'a [Box<[u8]>]);
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
        struct Contracts<'a>(&'a HashMap<String, Box<[u8]>>);

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
            .field("transactions", &Hexes(&*self.transactions))
            .finish()
    }
}

impl fmt::Debug for CollectionGuarantee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionGuarantee")
            .field("collection_id", &Hex(&self.collection_id))
            .field("signatures", &Hexes(&*self.signatures))
            .finish()
    }
}

impl fmt::Debug for TransactionResultResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionResultResponse")
            .field("status", &self.status)
            .field("status_code", &self.status_code)
            .field("error_message", &self.error_message)
            .field("events", &self.events)
            .field("block_id", &Hex(&self.block_id))
            .finish()
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Event");

        dbg.field("ty", &self.ty)
            .field("transaction_id", &Hex(&self.transaction_id))
            .field("transaction_index", &self.transaction_index)
            .field("event_index", &self.event_index);

        if let Ok(v) = self.parse_payload_as_value() {
            dbg.field("payload", &v);
        } else {
            dbg.field("payload", &String::from_utf8_lossy(&self.payload).as_ref());
        }

        dbg.finish()
    }
}

impl<A: AsRef<[u8]>> fmt::Debug for ProposalKeyE<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProposalKey")
            .field("address", &Hex(self.address.as_ref()))
            .field("key_id", &self.key_id)
            .field("sequence_number", &self.sequence_number)
            .finish()
    }
}

impl<A: AsRef<[u8]>, B: AsRef<[u8]>> fmt::Debug for SignatureE<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signature")
            .field("address", &Hex(self.address.as_ref()))
            .field("key_id", &self.key_id)
            .field("signature", &Hex(self.signature.as_ref()))
            .finish()
    }
}

struct Arguments<I>(I);

impl<I> fmt::Debug for Arguments<I>
where
    I: IntoIterator + Copy,
    I::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_list();

        for e in self.0 {
            f.entry(&String::from_utf8_lossy(e.as_ref()).as_ref());
        }

        f.finish()
    }
}

impl<Args> fmt::Debug for TransactionHeader<Args>
where
    for<'a> &'a Args: IntoIterator,
    for<'a> <&'a Args as IntoIterator>::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionHeader")
            .field("script", &self.script)
            .field("arguments", &Arguments(&self.arguments))
            .finish()
    }
}

impl<
        Script: AsRef<[u8]>,
        Arguments,
        ReferenceBlockId: AsRef<[u8]>,
        ProposalKeyAddress: AsRef<[u8]>,
        Payer: AsRef<[u8]>,
        Authorizers,
        PayloadSignatures,
        EnvelopeSignatures,
        PayloadSignatureAddress: AsRef<[u8]>,
        PayloadSignature: AsRef<[u8]>,
        EnvelopeSignatureAddress: AsRef<[u8]>,
        EnvelopeSignature: AsRef<[u8]>,
    > fmt::Debug
    for TransactionE<
        Script,
        Arguments,
        ReferenceBlockId,
        ProposalKeyAddress,
        Payer,
        Authorizers,
        PayloadSignatures,
        EnvelopeSignatures,
    >
where
    Arguments: HasItem,
    <Arguments as HasItem>::Item: AsRef<[u8]>,
    for<'a> &'a Arguments: IntoIterator<Item = &'a <Arguments as HasItem>::Item>,

    Authorizers: HasItem,
    <Authorizers as HasItem>::Item: AsRef<[u8]>,
    for<'a> &'a Authorizers: IntoIterator<Item = &'a <Authorizers as HasItem>::Item>,

    PayloadSignatures: HasItem<Item = SignatureE<PayloadSignatureAddress, PayloadSignature>>,
    for<'a> &'a PayloadSignatures:
        IntoIterator<Item = &'a SignatureE<PayloadSignatureAddress, PayloadSignature>>,

    EnvelopeSignatures: HasItem<Item = SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
    for<'a> &'a EnvelopeSignatures:
        IntoIterator<Item = &'a SignatureE<EnvelopeSignatureAddress, EnvelopeSignature>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        struct Signatures<'a, I>(I, PhantomData<&'a ()>);

        impl<'a, I, SigAddr: AsRef<[u8]> + 'a, Sig: AsRef<[u8]> + 'a> fmt::Debug for Signatures<'a, I>
        where
            I: IntoIterator<Item = &'a SignatureE<SigAddr, Sig>> + Copy,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0).finish()
            }
        }

        f.debug_struct("Transaction")
            .field(
                "script",
                &String::from_utf8_lossy(self.script.as_ref()).as_ref(),
            )
            .field("arguments", &Arguments(&self.arguments))
            .field("reference_block_id", &Hex(self.reference_block_id.as_ref()))
            .field("gas_limit", &self.gas_limit)
            .field("proposal_key", &self.proposal_key)
            .field("payer", &Hex(self.payer.as_ref()))
            .field("authorizers", &Hexes(&self.authorizers))
            .field(
                "payload_signatures",
                &Signatures(&self.payload_signatures, PhantomData),
            )
            .field(
                "envelope_signatures",
                &Signatures(&self.envelope_signatures, PhantomData),
            )
            .finish()
    }
}
