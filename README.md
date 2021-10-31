# Flow Rust SDK

[![CI](https://github.com/fee1-dead/flow.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fee1-dead/flow.rs/actions/workflows/ci.yml)

The minimum supported Rust version (MSRV) of this project is 1.56.0.

To install Rust, visit the [rustup] website for information.

## Examples

If you are looking for interactive examples, you can look at `flow-examples/`. Examples in the directory
runs an emulator and makes requests to the emulator.

To run examples in `flow-examples/`, make sure that you have [Flow CLI] installed and
run `cargo run -p flow-examples` in the project's root directory. I might ship prebuilt binaries for
flow-examples in the future.

Use `FLOW_CLI_EXTRA_ARGS` to control extra arguments passed to start the emulator. For example: use
`FLOW_CLI_EXTRA_ARGS="--http-port 8081"` if the 8080 port is in use.

If you are looking for examples that just fetches some information from the network without any input,
you can look at the `examples/` directory.

To run examples in `examples/`, run `cargo run --example file_name_without_rs`.

## Cryptography and Hashing algorithms

Currently the library supports signing with secp256k1 and SHA3 hashing. Support for other algorithms
may be added in the near future.

secp256k1 signing is provided by the `secp256k1` crate, which is an FFI wrapper around [libsecp256k1],
a C library by Pieter Wuille that is used in many bitcoin projects.

sha3 hashing is provided by the [`tiny-keccak`] crate, which claims to have [better performance] than
another crate by `RustCrypto`.

[rustup]: https://rustup.rs
[examples/]: ./examples/
[libsecp256k1]: https://github.com/bitcoin-core/secp256k1/
[`tiny-keccak`]: https://github.com/debris/tiny-keccak
[better performance]: https://github.com/debris/tiny-keccak#benchmarks
[Flow CLI]: https://docs.onflow.org/flow-cli/
