#![warn(missing_docs)]

//! # PubNub Rust SDK
//!
//!
//! <div align = "center">
//!
//! ![PubNub](https://raw.githubusercontent.com/pubnub/rust/phoenix/logo.svg)
//!
//! ![Tests](https://github.com/pubnub/rust/actions/workflows/run-tests.yml/badge.svg)
//! ![Validations](https://github.com/pubnub/rust/actions/workflows/run-validations.yml/badge.svg)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/pubnub/rust/LICENSE)
//!
//! **Make your app come alive with real-time experiences!**
//!
//! </div>
//!
//! ## Overview
//!
//! This is the official PubNub Rust SDK repository.
//!
//! [PubNub](https://www.pubnub.com/) takes care of the infrastructure and APIs needed for the realtime //! communication layer of your application. Work on your app's logic and let PubNub handle sending and receiving //! data across the world in less than 100ms.
//!
//! ## Getting started
//!
//! Below you can find everything you need to start messaging!
//!
//! ### Get PubNub keys
//!
//! You will need the publish and subscribe keys to authenticate your app. Get your keys from the [Admin Portal]//! (https://dashboard.pubnub.com/login).
//!
//! ### Import using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
//!
//! Add `pubnub` to your Rust project in the `Cargo.toml` file:
//!
//! ```toml
//! // default features
//! [dependencies]
//! pubnub = "0.0.0"
//!
//! // all features
//! [dependencies]
//! pubnub = { version = "0.0.0", features = ["full"] }
//! ```
//!
//! ### Example
//!
//! Try the following sample code to get up and running quicky!
//!
//! ```rust
//! // TODO
//! ```
//!
//! You can find more examples in our [examples](examples/) directory!
//!
//! ## Documentation
//!
//! * [API reference for Rust](TODO)
//!
//! ## Support
//!
//! If you **need help** or have a **general question**, contact support@pubnub.com.
//!
//! ## License
//!
//! This project is licensed under the [MIT license].
//!
//! [MIT license]: https://github.com/pubnub/LICENSE/blob/master/LICENSE
//!

pub use self::core::PubNubError;
pub use self::core::Transport;
pub use self::core::TransportRequest;
pub use self::core::TransportResponse;
pub mod core;

pub mod dx;
pub mod providers;
pub mod transport;
