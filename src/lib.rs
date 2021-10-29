//! ## Flow.rs
//!
//! This crate contains various ways to interact with the Flow blockchain through its access API.
//!
//! Start by connecting to the [`testnet()`] or the [`mainnet()`], or [login to an account] by
//! providing the address and secret key.
//!
//! [`testnet()`]: crate::client::TonicHyperFlowClient::testnet()
//! [`mainnet()`]: crate::client::TonicHyperFlowClient::mainnet()
//! [login to an account]: crate::account::Account::new()

pub mod access;

pub mod client;
pub mod codec;

pub mod protobuf;

pub mod transaction;

pub mod entities;

pub mod algorithms;
pub mod requests;

pub mod account;
pub mod multi;
pub mod sign;

pub mod prelude;

#[cfg(test)]
pub mod tests;

mod fmt;
