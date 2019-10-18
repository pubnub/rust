# PubNub Rust SDK

The PubNub Rust SDK is based on Tokio Asyn `0.2`.

Since we are using Async/Await, a feature soon to be relased,
we need to update the version of rust installed.

```shell
rustup toolchain install nightly
rustup default nightly
```

The lib successfully builds on `rustc 1.40.0-nightly (fa0f7d008 2019-10-17)`.
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

### Rust Docss

```shell
cargo doc
```

### Run Tests
```shell
cargo test
```
