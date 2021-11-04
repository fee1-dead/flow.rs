//! Documentation testing.
//!
//! Tests examples inside `/docs/README.md`.

#[doc = include_str!("../docs/README.md")]
pub struct ReferenceDocTest;
