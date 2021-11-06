//! Test that various parts of the crate can have mock implementations,
//! and thus, allows other valid implementations than the default implementations.

pub mod account;
pub mod algorithms;
pub mod client;

use futures_util::FutureExt;

/// Poll on a future, assuming that it already has a value.
pub fn immediate_fut<F: std::future::Future>(future: F) -> F::Output {
    future.now_or_never().unwrap()
}
