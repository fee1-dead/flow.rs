//! ## Flow.rs
//!
//! This crate contains various ways to interact with the Flow blockchain through its access API.
//!
//! Start by connecting to the [`testnet()`] or the [`mainnet()`], or [login to an account] by
//! providing the address and secret key.
//!
//! You can also take a look at [the reference] for examples around individual functionalities.
//!
//! [`testnet()`]: crate::client::TonicHyperFlowClient::testnet()
//! [`mainnet()`]: crate::client::TonicHyperFlowClient::mainnet()
//! [login to an account]: crate::account::Account::new()
//! [the reference]: https://github.com/fee1-dead/flow.rs/blob/master/docs/README.md
#![forbid(missing_docs)] // Every public item must be documented

pub mod prelude;

pub mod access;
pub mod account;
pub mod algorithms;
pub mod client;
pub mod codec;
pub mod entities;
pub mod error;
pub mod multi;
pub mod protobuf;
pub mod requests;
pub mod sign;
pub mod transaction;

#[cfg(test)]
pub mod tests;

#[cfg(doctest)]
pub mod doctests;

mod fmt;
