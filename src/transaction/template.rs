use std::{borrow::Cow, collections::HashMap};

use cadence_json::{EntryRef, UFix64, ValueRef};
use serde::Serialize;

use crate::multi::SigningParty;
use crate::client::GrpcClient;
use crate::algorithms::*;
use crate::access::{BlockHeaderResponse, GetLatestBlockHeaderRequest};
use crate::protobuf::Seal;

/// A `TransactionHeader` is a template plus arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionHeader<Arguments> {
    /// The script, provided by the template.
    pub script: Cow<'static, str>,
    /// Encoded arguments.
    ///
    /// The number of arguments is determined by the template.
    pub arguments: Arguments,
}

impl<Arguments> TransactionHeader<Arguments> {
    pub async fn into_party<C>(
        self,
        client: &mut C,
        gas_limit: u64,
        proposer_address: impl Into<Box<[u8]>>,
        proposal_key_id: u64,
        proposal_key_sequence_number: u64,
        payer: impl Into<Box<[u8]>>,
        authorizers: impl IntoIterator<Item = impl Into<Box<[u8]>>>,
    ) -> Result<SigningParty, C::Error>
    where
        C: GrpcClient<GetLatestBlockHeaderRequest, BlockHeaderResponse>,
        Arguments: Into<Box<[Box<[u8]>]>>,
    {
        let arguments = self.arguments.into();
        let reference_id = client.send(GetLatestBlockHeaderRequest { seal: Seal::Sealed  }).await?.0.id;
        let proposer_address = proposer_address.into();
        let payer = payer.into();
        let authorizers = authorizers.into_iter().map(Into::into).collect();
        Ok(SigningParty::new(
            self.script.into_owned().into_boxed_str(),
            arguments,
            reference_id.into(),
            gas_limit,
            proposer_address,
            proposal_key_id,
            proposal_key_sequence_number,
            payer,
            authorizers,
        ))
    }
}

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
///     script: SCRIPT.as_bytes().into(),
///     arguments: vec![serde_json::to_vec(&argument).unwrap().into_boxed_slice()]
/// })
/// ```
impl TransactionHeaderBuilder {
    #[inline]
    pub const fn new() -> Self {
        Self {
            script: None,
            arguments: Vec::new(),
        }
    }

    #[inline]
    pub fn script_static<B: ?Sized + AsRef<str>>(mut self, script: &'static B) -> Self {
        self.script = Some(Cow::Borrowed(script.as_ref()));
        self
    }

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

    #[inline]
    pub fn argument<'a>(mut self, val: impl AsRef<ValueRef<'a>>) -> Self {
        self.arguments
            .push(serde_json::to_vec(val.as_ref()).unwrap().into_boxed_slice());
        self
    }

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

    #[inline]
    pub fn build(self) -> TransactionHeader<Vec<Box<[u8]>>> {
        TransactionHeader {
            script: self.script.unwrap(),
            arguments: self.arguments,
        }
    }

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
    pub public_keys: &'a [PubKey],
}

#[derive(Clone, Copy)]
pub struct CreateAccountWeightedTransaction<'a, PubKey> {
    pub public_key: &'a [(PubKey, UFix64)],
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

impl<PubKey> CreateAccountTransaction<'_, PubKey> {
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
    pub fn to_header<S: FlowSigner<PublicKey = PubKey>, H: FlowHasher>(&self, signer: &S) -> TransactionHeader<[Box<[u8]>; 1]> {
        let script = format!(
            include_str!("create_account_weighted.cdc.template"),
            S::Algorithm::NAME,
            H::Algorithm::NAME
        );
        let entries: Vec<_> = self.public_key.iter().map(|(key, seqnum)| (hex::encode(signer.serialize_public_key(key)), seqnum)).collect();
        let dict_entries: Vec<_> = entries.iter().map(|(key, seqnum)| EntryRef { key: ValueRef::String(key), value: ValueRef::UFix64(**seqnum) }).collect();
        let dict = ValueRef::Dictionary(&dict_entries);

        header_array(script.into(), [dict])
    }
}

impl<Name: AsRef<str>, Script: AsRef<str>> AddContractTransaction<'_, Name, Script> {
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
