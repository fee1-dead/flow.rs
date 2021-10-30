//! Re-exports for commonly used types and functions.

pub use crate::account::Account;
pub use crate::account::DefaultAccount;
pub use crate::algorithms::{DefaultHasher, DefaultSigner};

#[cfg(feature = "tonic-transport")]
pub use crate::client::TonicHyperFlowClient;

pub use crate::protobuf::Seal;
pub use crate::transaction::{
    AddContractTransaction, CreateAccountTransaction, CreateAccountWeightedTransaction,
    RemoveContractTransaction, TransactionHeaderBuilder, UpdateContractTransaction,
};

/// Re-exports items from the cadence_json crate.
pub mod cadence_json {
    pub use cadence_json::*;
}

/// Re-exports items from the hex crate.
pub mod hex {
    pub use hex::*;
}