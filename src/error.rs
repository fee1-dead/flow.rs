//! Errors

use std::{error::Error, fmt};

/// Boxed Error.
pub struct BoxError(Box<dyn Error + Send + Sync>);

/// The errors that could happen when logging in an account.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AccountError {
    /// There was not any public keys that matched the provided private key.
    #[error("Could not find a matching key for the private key.")]
    NoMatchingKeyFound,

    /// The hashing and signing algorithms provided did not match public information.
    #[error("The hashing and signing algorithms do not match.")]
    AlgoMismatch,

    /// Did not have enough signing weight.
    #[error("The key(s) does not have enough weight.")]
    NotEnoughWeight,

    /// A key provided was revoked.
    #[error("A key was revoked.")]
    KeyRevoked,

    /// An error from the client has occured.
    #[error(transparent)]
    Custom(#[from] Box<dyn Error + Send + Sync>),
}

/// The errors that could happen when sending a request via tonic.
#[derive(Debug, thiserror::Error)]
pub enum TonicError {
    /// A gRPC status describing the result of an RPC call.
    #[error(transparent)]
    Status(#[from] tonic::Status),

    /// Custom error
    #[error(transparent)]
    Custom(#[from] Box<dyn Error + Send + Sync>),
}

impl From<Box<dyn Error + Send + Sync>> for BoxError {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        Self(e)
    }
}

impl fmt::Debug for BoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for BoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Error for BoxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}
