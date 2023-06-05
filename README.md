# PubNub Rust SDK

<div align = "center">

![PubNub](https://raw.githubusercontent.com/pubnub/rust/phoenix/logo.svg)

![Tests](https://github.com/pubnub/rust/actions/workflows/run-tests.yml/badge.svg)
![Validations](https://github.com/pubnub/rust/actions/workflows/run-validations.yml/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/pubnub/rust/LICENSE)

**Make your app come alive with real-time experiences!**

</div>

## Overview

This is the official PubNub Rust SDK repository.

[PubNub](https://www.pubnub.com/) takes care of the infrastructure and APIs needed for the realtime
communication layer of your application. Work on your app's logic and let PubNub handle sending and receiving
data across the world in less than 100ms.

## Getting started

Below you can find everything you need to start messaging!

### Get PubNub keys

You will need the publish and subscribe keys to authenticate your app. Get your keys from the [Admin Portal](https://dashboard.pubnub.com/login).

### Import using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Add `pubnub` to your Rust project in the `Cargo.toml` file:

```toml
# default features
[dependencies]
pubnub = "0.0.0"

# all features
[dependencies]
pubnub = { version = "0.0.0", features = ["full"] }
```

### Example

Try the following sample code to get up and running quickly!

```rust
use pubnub::{Keyset, PubNubClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let publish_key = "my_publish_key";
    let subscribe_key = "my_subscribe_key";

    let client = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    client
        .publish_message("hello world!")
        .channel("my_channel")
        .r#type("text-message")
        .execute()
        .await?;

    Ok(())
}
```

You can find more examples in our [examples](examples/) directory!

## Features

The `pubnub` crate is split into multiple features. You can enable or disable them in the `Cargo.toml` file, like so:

```toml
# only blocking and access + default features
[dependencies]
pubnub = { version = "0.0.0", features = ["blocking", "access"] }

# only parse_token + default features
[dependencies]
pubnub = { version = "0.0.0", features = ["parse_token"] }
```

### Available features

| Feature name  | Description | Available PubNub APIs |
| :------------ | :---------- | :------------- |
| `full`        | Enables all non-conflicting features | Configuration, Publish, Access Manager, Parse Token |
| `default`     | Enables default features: `publish`, `serde`, `reqwest`, `aescbc`, `std` | Configuration, Publish |
| `publish`     | Enables Publish API | Configuration, Publish |
| `access`      | Enables Access Manager API | Configuration, Access Manager |
| `parse_token` | Enables parsing Access Manager tokens | Configuration, Parse Token |
| `serde`       | Uses [serde](https://github.com/serde-rs/serde) for serialization | n/a |
| `reqwest`     | Uses [reqwest](https://github.com/seanmonstar/reqwest) as a transport layer | n/a |
| `blocking`    | Enables blocking executions of APIs | n/a |
| `aescbc`      | Enables AES-CBC encryption | n/a |
| `std`         | Enables `std` library | n/a |

## Documentation

* [API reference for Rust](https://www.pubnub.com/docs/sdks/rust)
* [Rust docs](https://www.docs.rs/pubnub/latest/pubnub)

## Wasm support

The `pubnub` crate is compatible with WebAssembly. You can use it in your Wasm project.

## `no_std` support

The `pubnub` crate is `no_std` compatible. To use it in a `no_std` environment, you have to disable the default
features and enable the ones you need, for example:

```toml
[dependencies]
pubnub = { version = "0.0.0", default-features = false, features = ["serde", "publish",
"blocking"] }
```

### Limitations

The `no_std` support is limited by the implementation details of the SDK.

The SDK uses the `alloc` crate to allocate memory for some operations, which means that
certain targets aren't supported. Additionally, as we provide a synchronous API, we use
some parts of the `alloc::sync` module, which is also not supported in certain `no_std` environments.

Some SDK features aren't supported in a `no_std` environment:

* partially `access` module (because of lack timestamp support)
* partially `reqwest` transport (because of the reqwest implementation details)
* `std` feature (because of the `std` library)

We depend on a random number generator to generate data for debugging purposes.
If you want to use the SDK in a `no_std` environment, you'll have to provide
your own random number generator implementation for certain targets.

See more:

* [`getrandom` crate](https://docs.rs/getrandom/latest/getrandom/)
* [no_std examples](https://github.com/pubnub/rust/tree/master/examples/no_std/)

If you're having problems compiling this crate for more exotic targets, you can try to use the
`extra_platforms` feature. Be aware that this feature is **not supported** and we do not recommend using it.

For more information about this feature. refer to [Cargo.toml](Cargo.toml) in the `[features]` section.

## Support

If you **need help** or have a **general question**, contact support@pubnub.com.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/pubnub/LICENSE/blob/master/LICENSE

