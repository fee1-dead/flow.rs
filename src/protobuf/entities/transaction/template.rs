use std::{array::IntoIter, borrow::Cow, iter};

use cadence_json::ValueRef;

use crate::algorithms::*;

/// A `TransactionHeader` is a template plus arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionHeader<Arguments> {
    /// The script, provided by the template.
    pub script: Cow<'static, [u8]>,
    /// An array of encoded arguments.
    ///
    /// The number of arguments is determined by the template.
    pub arguments: Arguments,
}

/// A simple transaction to create an account with full weight keys.
pub struct CreateAccountTransaction<'a, PubKey> {
    pub public_keys: &'a [PubKey],
}

pub struct AddContractTransaction<Name: AsRef<str>, Script: AsRef<str>, ExtraArgs> {
    pub name: Name,
    pub script: Script,
    pub extra_args: ExtraArgs,
}

impl<Name: AsRef<str>, Script: AsRef<str>, ExtraArgs>
    AddContractTransaction<Name, Script, ExtraArgs>
{
}

impl<'a, PubKey> CreateAccountTransaction<'a, PubKey> {
    pub fn to_header<S: FlowSigner<PublicKey = PubKey>, H: FlowHasher>(
        &self,
        signer: &S,
    ) -> TransactionHeader<[Vec<u8>; 1]> {
        match self.public_keys {
            [pubkey] => {
                let bytes = signer.serialize_public_key(pubkey);
                let bytes = bytes.map(ValueRef::UInt8);
                let script = format!(
                    include_str!("create_account_one_key.cdc.template"),
                    S::Algorithm::NAME,
                    H::Algorithm::NAME
                );
                header_array(script.into_bytes().into(), [ValueRef::Array(&bytes)])
            }
            pubs => {
                let arrays = pubs
                    .iter()
                    .map(|pubkey| signer.serialize_public_key(pubkey))
                    .map(|bytes| bytes.map(ValueRef::UInt8))
                    .collect::<Vec<_>>();

                let args = arrays
                    .iter()
                    .map(AsRef::as_ref)
                    .map(ValueRef::Array)
                    .collect::<Vec<_>>();

                let val = ValueRef::Array(&args);

                header_array(
                    format!(
                        include_str!("create_account.cdc.template"),
                        S::Algorithm::NAME,
                        H::Algorithm::NAME
                    )
                    .into_bytes()
                    .into(),
                    [val],
                )
            }
        }
    }
}

fn header_array<'a, const ARGS: usize>(
    script: Cow<'static, [u8]>,
    args: [ValueRef; ARGS],
) -> TransactionHeader<[Vec<u8>; ARGS]> {
    TransactionHeader {
        script,
        arguments: args.map(|s| serde_json::to_vec(&s).unwrap()),
    }
}
