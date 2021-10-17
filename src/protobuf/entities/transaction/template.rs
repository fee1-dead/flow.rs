use std::{borrow::Cow, collections::HashMap};

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

pub struct AddContractTransaction<'a, Name: AsRef<str>, Script: AsRef<str>> {
    pub name: Name,
    pub script: Script,
    pub extra_args: HashMap<String, ValueRef<'a>>,
}

pub struct UpdateContractTransaction<Name: AsRef<str>, Script: AsRef<str>> {
    pub name: Name,
    pub script: Script,
}

pub struct RemoveContractTransaction<Name: AsRef<str>> {
    pub name: Name,
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


impl<Name: AsRef<str>, Script: AsRef<str>> AddContractTransaction<'_, Name, Script> {
    pub fn to_header(&self) -> TransactionHeader<Vec<Vec<u8>>> {
        // Extra args passed to the transaction.
        // name: type, name: type, ...
        let mut extra_args_transaction_args = String::new();
        // Extra args passed to contracts.add().
        // name: name, name: name, ...
        let mut extra_args_add_args = String::new();

        let base_arguments = [
            ValueRef::String(self.name.as_ref()),
            ValueRef::String(self.script.as_ref()),
        ];

        let extra_args = self.extra_args.iter().enumerate().map(|(n, (name, val))| {
            if n != 0 {
                extra_args_transaction_args.push_str(", ");
                extra_args_add_args.push_str(", ")
            }
            extra_args_transaction_args.push_str(name);
            extra_args_transaction_args.push_str(": ");
            extra_args_transaction_args.push_str(val.ty().as_str());

            extra_args_add_args.push_str(name);
            extra_args_add_args.push_str(": ");
            extra_args_add_args.push_str(name);

            val
        });

        let arguments = base_arguments
            .iter()
            .chain(extra_args)
            .map(|v| serde_json::to_vec(v).unwrap())
            .collect();

        TransactionHeader {
            script: format!(
                include_str!("add_contract.cdc.template"),
                extra_args_transaction_args, extra_args_add_args
            )
            .into_bytes()
            .into(),
            arguments,
        }
    }
}

impl<Name: AsRef<str>, Script: AsRef<str>> UpdateContractTransaction<Name, Script> {
    pub fn to_header(&self) -> TransactionHeader<[Vec<u8>; 2]> {
        header_array(
            include_str!("update_contract.cdc").as_bytes().into(),
            [
                ValueRef::String(self.name.as_ref()),
                ValueRef::String(self.script.as_ref()),
            ],
        )
    }
}

impl<Name: AsRef<str>> RemoveContractTransaction<Name> {
    pub fn to_header(&self) -> TransactionHeader<[Vec<u8>; 1]> {
        header_array(
            include_str!("remove_contract.cdc").as_bytes().into(),
            [
                ValueRef::String(self.name.as_ref())
            ],
        )
    }
}

fn header_array<const ARGS: usize>(
    script: Cow<'static, [u8]>,
    args: [ValueRef; ARGS],
) -> TransactionHeader<[Vec<u8>; ARGS]> {
    TransactionHeader {
        script,
        arguments: args.map(|s| serde_json::to_vec(&s).unwrap()),
    }
}
