//! Trait bound HACK.
//!
//! [#89196](https://github.com/rust-lang/rust/issues/89196) makes HRTBs like this unusable:
//!
//! ```
//! # fn some_fn<T>(it: T)
//! where
//!     for<'a> &'a T: IntoIterator,
//!     for<'a> <&'a T as IntoIterator>::IntoIter: Clone,
//! # {}
//! ```
//!
//! While wrapping it inside a wrapper would work:
//!
//! ```
//! # struct Hack<T>(T);
//! # fn some_fn<T>(it: T)
//! where
//!     for<'a> &'a T: IntoIterator,
//!     for<'a> Hack(<&'a T as IntoIterator>::IntoIter): Clone,
//! # {}
//! ```
//!
//! ## Prior Art
//!
//! The `yoke` crate currently [uses](https://docs.rs/yoke/0.3.1/yoke/trait_hack/struct.YokeTraitHack.html) this hack.

#[repr(transparent)]
#[derive(Clone)]
pub struct Hack<T>(pub T);
