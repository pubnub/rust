# PubNub Rust SDK

The PubNub Rust SDK is based on Tokio Asyn `0.2`.

Since we are using Async/Await, a feature soon to be relased,
we need to update the version of rust installed.

``
rustup toolchain install nightly
rustup default nightly
```

We will continue to update the version definitions as they become available.

### Trying Examples

You can find these examples in the `./examples` directory.
Explore the usage patterns available in each of the examples.
You can easily run each of the examples with these cargo commands:

```
cargo run --example publish
```

```
cargo run --example publish-subscribe
```

```
cargo run --example presence
```

### Rust Docss

```
cargo doc
```

### Run Tests
```
cargo test
```
