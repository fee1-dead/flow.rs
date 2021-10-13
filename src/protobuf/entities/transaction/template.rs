use std::{borrow::Cow, marker::PhantomData};

use cadence_json::ValueRef;

use crate::algorithms::*;

/// A `TransactionHeader` is a template plus arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionHeader<const ARGS: usize> {
    /// The script, provided by the template.
    pub script: Cow<'static, [u8]>,
    /// An array of encoded arguments.
    ///
    /// The number of arguments is determined by the template.
    pub arguments: [Vec<u8>; ARGS],
}

/// A simple transaction to create an account with full weight keys.
pub struct CreateAccountTransaction<'a, Signer: FlowSigner, Hasher> {
    pub public_keys: &'a [Signer::PublicKey],
    pub signer: &'a Signer,
    pub _pd: PhantomData<Hasher>,
}

impl<'a, S: FlowSigner, H: FlowHasher> CreateAccountTransaction<'a, S, H> {
    pub fn to_header(&self) -> TransactionHeader<1> {
        match self.public_keys {
            [pubkey] => {
                let bytes = self.signer.serialize_public_key(pubkey);
                let bytes = bytes.map(ValueRef::UInt8);
                let script = format!(include_str!("create_account_one_key.cdc.template"), S::Algorithm::NAME, H::Algorithm::NAME);
                header(script.into_bytes().into(), [ValueRef::Array(&bytes)])
            }
            pubs => {
                let arrays = pubs
                    .iter()
                    .map(|pubkey| self.signer.serialize_public_key(pubkey))
                    .map(|bytes| bytes.map(ValueRef::UInt8))
                    .collect::<Vec<_>>();

                let args = arrays
                    .iter()
                    .map(AsRef::as_ref)
                    .map(ValueRef::Array)
                    .collect::<Vec<_>>();

                let val = ValueRef::Array(&args);

                header(format!(include_str!("create_account.cdc.template"), S::Algorithm::NAME, H::Algorithm::NAME).into_bytes().into(), [val])
            }
        }
    }
}

fn header<const ARGS: usize>(
    script: Cow<'static, [u8]>,
    args: [ValueRef; ARGS],
) -> TransactionHeader<ARGS> {
    let arguments = args.map(|v| serde_json::to_vec(&v).unwrap());
    TransactionHeader { script, arguments }
}
