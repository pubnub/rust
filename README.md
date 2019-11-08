# PubNub Rust SDK

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

The PubNub Rust SDK is based on Tokio `0.2`.
This library uses `HTTP/2.0` to communicate with the PubNub Edge Messaging Network.

Since we are using Async/Await, a feature soon to be released,
we need to update the version of rust installed.

```shell
rustup toolchain install stable
rustup default stable
```

The lib successfully builds on `rustc 1.39.0 (4560ea788 2019-11-04)`.
We will continue to update the version definitions as they become available.

## Get Started

First you'll need to add the dependency to your project.

### Cargo.toml

Easily add PubNub lib to your `Cargo.toml` file.

```toml
[dependencies.pubnub]
git = "https://github.com/stephenlb/pubnub-rust"
```

### Simple Usage

Try the following sample code to get up and running quickly.

```rust
// TODO
```

### Trying Examples

You can find these examples in the `./examples` directory.
Explore the usage patterns available in each of the examples.
You can easily run each of the examples with these cargo commands:

```shell
cargo run --example publish
```

```shell
cargo run --example publish-subscribe
```

```shell
cargo run --example presence
```

### Rust Docs

```shell
cargo doc
```

### Run Tests
```shell
cargo test
```