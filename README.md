# Flow Rust SDK

[![CI](https://github.com/fee1-dead/flow.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fee1-dead/flow.rs/actions/workflows/ci.yml)

The minimum supported Rust version (MSRV) of this project is 1.56.0.

To install Rust, visit the [rustup] website for information.

## Examples

Examples are in the [examples/] directory. To run an example, e.g. `examples/helloworld.rs`,
run `cargo +nightly run --example helloworld`.

## Cryptography and Hashing algorithms

Currently the library supports signing with secp256k1 and SHA3 hashing. Support for other algorithms
may be added in the near future.

secp256k1 signing is provided by the `secp256k1` crate, which is an FFI wrapper around [libsecp256k1],
a C library by Pieter Wuille that is used in many bitcoin projects.

sha3 hashing is provided by the [`tiny-keccak`] crate, which claims to have [better performance] than
another crate by `RustCrypto`.

[rustup]: rustup.rs
[examples/]: ./examples/
[libsecp256k1]: https://github.com/bitcoin-core/secp256k1/
[`tiny-keccak`]: https://github.com/debris/tiny-keccak
[better performance]: https://github.com/debris/tiny-keccak#benchmarks
