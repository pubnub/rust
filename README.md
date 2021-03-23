# PubNub Rust SDK

[![Build Status](https://travis-ci.com/pubnub/rust.svg?branch=master)](https://travis-ci.com/pubnub/rust)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

The PubNub Rust SDK is based on Tokio `1.0`. This library uses `HTTP/2` to communicate with the PubNub Edge Messaging Network.

## MSRV

Supports Rust 1.39.0 and higher.

## Get Started

First you'll need to add the dependency to your project.

### Cargo.toml

Easily add PubNub lib to your `Cargo.toml` file.

```toml
[dependencies.pubnub_hyper]
git = "https://github.com/pubnub/rust"
```

## Simple Usage

Try the following sample code to get up and running quickly.

```rust
use futures_util::stream::StreamExt;
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::{core::json::object, Builder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = Hyper::new()
        .publish_key("demo")
        .subscribe_key("demo")
        .build()?;

    let mut pubnub = Builder::new()
        .transport(transport)
        .runtime(TokioGlobal)
        .build();

    let message = object! {
        "username" => "JoeBob",
        "content" => "Hello, world!",
    };

    let mut stream = pubnub.subscribe("my-channel").await;
    let timetoken = pubnub.publish("my-channel", message).await?;
    println!("timetoken = {:?}", timetoken);

    let received = stream.next().await;
    println!("received = {:?}", received);

    Ok(())
}
```

## Examples

You can find these examples in the `./pubnub-hyper/examples` directory.
Explore the usage patterns available in each of the examples.
You can easily run each of the examples with these cargo commands:

```shell
cargo run --example publish-subscribe
```
