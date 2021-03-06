use std::borrow::Cow;
use std::collections::HashMap;

use cadence_json::{EntryRef, UFix64, ValueRef};
use serde::Serialize;

use crate::algorithms::*;
use crate::multi::PartyBuilder;

/// A `TransactionHeader` is a template plus arguments.
#[derive(Clone, PartialEq, Eq)]
pub struct TransactionHeader<Arguments> {
    /// The script, provided by the template.
    pub script: Cow<'static, str>,
    /// Encoded arguments.
    ///
    /// The number of arguments is determined by the template.
    pub arguments: Arguments,
}

impl<Arguments> TransactionHeader<Arguments> {
    /// Creates a [`PartyBuilder`] and feeds it the transaction header's
    /// script and arguments.
    pub fn into_party_builder(self) -> PartyBuilder
    where
        Arguments: IntoIterator<Item = Box<[u8]>>,
    {
        PartyBuilder::new()
            .script(self.script.into_owned())
            .arguments_raw(self.arguments)
    }
}

/// A builder for [`TransactionHeader`]s.
#[derive(Default)]
pub struct TransactionHeaderBuilder {
    script: Option<Cow<'static, str>>,
    arguments: Vec<Box<[u8]>>,
}

/// A builder for a transaction header.
///
/// ```rust
/// use cadence_json::ValueRef;
/// # use flow_sdk::transaction::{TransactionHeader, TransactionHeaderBuilder};
/// const SCRIPT: &str = r#"
///     transaction(greeting: String) {
///        let guest: Address
///
///        prepare(authorizer: AuthAccount) {
///            self.guest = authorizer.address
///        }
///
///        execute {
///            log(greeting.concat(",").concat(guest.toString()))
///        }
///     }
/// "#;
///
/// let argument = ValueRef::String("Hello");
///
/// let header = TransactionHeaderBuilder::new().script_static(SCRIPT).argument(&argument);
///
/// assert_eq!(header.build(), TransactionHeader {
///     script: SCRIPT.into(),
///     arguments: vec![serde_json::to_vec(&argument).unwrap().into_boxed_slice()]
/// })
/// ```
impl TransactionHeaderBuilder {
    /// Creates a new builder for [`TransactionHeader`]s.
    #[inline]
    pub const fn new() -> Self {
        Self {
            script: None,
            arguments: Vec::new(),
        }
    }

    /// Sets the script of the transaction.
    #[inline]
    pub fn script_static<B: ?Sized + AsRef<str>>(mut self, script: &'static B) -> Self {
        self.script = Some(Cow::Borrowed(script.as_ref()));
        self
    }

    /// Sets the script of the transaction.
    #[inline]
    pub fn script_owned(mut self, script: String) -> Self {
        self.script = Some(Cow::Owned(script));
        self
    }

    /// Clone the script into the builder. Do not use this if you have owned instances or static reference.
    #[inline]
    pub fn script_shared(mut self, script: impl AsRef<str>) -> Self {
        self.script = Some(Cow::Owned(script.as_ref().to_owned()));
        self
    }

    /// Adds an argument to the transaction header.
    #[inline]
    pub fn argument<'a>(mut self, val: impl AsRef<ValueRef<'a>>) -> Self {
        self.arguments
            .push(serde_json::to_vec(val.as_ref()).unwrap().into_boxed_slice());
        self
    }

    /// Adds arguments to the transaction header.
    #[inline]
    pub fn arguments<I>(mut self, args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Serialize,
    {
        self.arguments.extend(
            args.into_iter()
                .map(|v| serde_json::to_vec(&v))
                .map(Result::unwrap)
                .map(Vec::into_boxed_slice),
        );
        self
    }

    /// Builds a transaction header, assuming the script was set.
    #[inline]
    pub fn build(self) -> TransactionHeader<Vec<Box<[u8]>>> {
        TransactionHeader {
            script: self.script.unwrap(),
            arguments: self.arguments,
        }
    }

    /// Shorthand for `self.build().into_party_builder()`.
    #[inline]
    pub fn into_party_builder(self) -> PartyBuilder {
        self.build().into_party_builder()
    }

    /// Builds a transaction header, returning Err(self) if the script was not set.
    #[inline]
    pub fn build_checked(self) -> Result<TransactionHeader<Vec<Box<[u8]>>>, Self> {
        match self.script {
            Some(script) => Ok(TransactionHeader {
                script,
                arguments: self.arguments,
            }),
            None => Err(self),
        }
    }
}

/// A simple transaction to create an account with full weight keys.
#[derive(Clone, Copy)]
pub struct CreateAccountTransaction<'a, PubKey> {
    /// The public keys of the new account. The keys will have full signing weight.
    pub public_keys: &'a [PubKey],
}

/// A transaction to create an account with weighed keys.
#[derive(Clone, Copy)]
pub struct CreateAccountWeightedTransaction<'a, PubKey> {
    /// The public keys and their weight of the new account.
    pub public_keys: &'a [(PubKey, UFix64)],
}

/// Adds a contract to an account.
pub struct AddContractTransaction<'a, Name: AsRef<str>, Script: AsRef<str>> {
    /// The name of the contract.
    pub name: Name,

    /// The script of the contract.
    pub script: Script,

    /// The extra arguments passed to the contract's initialization.
    pub extra_args: HashMap<String, ValueRef<'a>>,
}

/// Updates a contract of an account by name.
pub struct UpdateContractTransaction<Name: AsRef<str>, Script: AsRef<str>> {
    /// The name of the contract to be updated.
    pub name: Name,

    /// The updated script of the contract.
    pub script: Script,
}

/// Remove a contract of an account by name.
pub struct RemoveContractTransaction<Name: AsRef<str>> {
    /// The name of the contract to be removed.
    pub name: Name,
}

impl<PubKey> CreateAccountTransaction<'_, PubKey> {
    /// Turns this transaction template into a [`TransactionHeader`].
    ///
    /// Requires the signer to serialize the public keys,
    /// and the hasher type argument to obtain the hashing algorithm.
    pub fn to_header<S: FlowSigner<PublicKey = PubKey>, H: FlowHasher>(
        &self,
        signer: &S,
    ) -> TransactionHeader<[Box<[u8]>; 1]> {
        match self.public_keys {
            [pubkey] => {
                let bytes = signer.serialize_public_key(pubkey);
                let bytes = bytes.map(ValueRef::UInt8);
                let script = format!(
                    include_str!("create_account_one_key.cdc.template"),
                    S::Algorithm::NAME,
                    H::Algorithm::NAME
                );
                header_array(script.into(), [ValueRef::Array(&bytes)])
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
                    .into(),
                    [val],
                )
            }
        }
    }
}

impl<PubKey> CreateAccountWeightedTransaction<'_, PubKey> {
    /// Turns this transaction template into a [`TransactionHeader`].
    ///
    /// Requires the signer to serialize the public keys,
    /// and the hasher type argument to obtain the hashing algorithm.
    pub fn to_header<S: FlowSigner<PublicKey = PubKey>, H: FlowHasher>(
        &self,
        signer: &S,
    ) -> TransactionHeader<[Box<[u8]>; 1]> {
        let script = format!(
            include_str!("create_account_weighted.cdc.template"),
            S::Algorithm::NAME,
            H::Algorithm::NAME
        );
        let entries: Vec<_> = self
            .public_keys
            .iter()
            .map(|(key, seqnum)| (hex::encode(signer.serialize_public_key(key)), seqnum))
            .collect();
        let dict_entries: Vec<_> = entries
            .iter()
            .map(|(key, seqnum)| EntryRef {
                key: ValueRef::String(key),
                value: ValueRef::UFix64(**seqnum),
            })
            .collect();
        let dict = ValueRef::Dictionary(&dict_entries);

        header_array(script.into(), [dict])
    }
}

impl<Name: AsRef<str>, Script: AsRef<str>> AddContractTransaction<'_, Name, Script> {
    /// Turns this transaction template into a [`TransactionHeader`].
    pub fn to_header(&self) -> TransactionHeader<Vec<Box<[u8]>>> {
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
            .map(|v| serde_json::to_vec(v).unwrap().into_boxed_slice())
            .collect();

        TransactionHeader {
            script: format!(
                include_str!("add_contract.cdc.template"),
                extra_args_transaction_args, extra_args_add_args
            )
            .into(),
            arguments,
        }
    }
}

impl<Name: AsRef<str>, Script: AsRef<str>> UpdateContractTransaction<Name, Script> {
    /// Turns this transaction template into a [`TransactionHeader`].
    pub fn to_header(&self) -> TransactionHeader<[Box<[u8]>; 2]> {
        header_array(
            include_str!("update_contract.cdc").into(),
            [
                ValueRef::String(self.name.as_ref()),
                ValueRef::String(self.script.as_ref()),
            ],
        )
    }
}

impl<Name: AsRef<str>> RemoveContractTransaction<Name> {
    /// Turns this transaction template into a [`TransactionHeader`].
    pub fn to_header(&self) -> TransactionHeader<[Box<[u8]>; 1]> {
        header_array(
            include_str!("remove_contract.cdc").into(),
            [ValueRef::String(self.name.as_ref())],
        )
    }
}

fn header_array<const ARGS: usize>(
    script: Cow<'static, str>,
    args: [ValueRef; ARGS],
) -> TransactionHeader<[Box<[u8]>; ARGS]> {
    TransactionHeader {
        script,
        arguments: args.map(|s| serde_json::to_vec(&s).unwrap().into_boxed_slice()),
    }
}
