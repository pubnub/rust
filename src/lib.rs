#![warn(missing_docs)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

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
//! [PubNub](https://www.pubnub.com/) takes care of the infrastructure and APIs needed for the realtime
//! communication layer of your application. Work on your app's logic and let PubNub handle sending and receiving
//! data across the world in less than 100ms.
//!
//! ## Getting started
//!
//! Below you can find everything you need to start messaging!
//!
//! ### Get PubNub keys
//!
//! You will need the publish and subscribe keys to authenticate your app. Get your keys from the [Admin Portal](https://dashboard.pubnub.com/login).
//!
//! ### Import using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
//!
//! Add `pubnub` to your Rust project in the `Cargo.toml` file:
//!
//! ```toml
//! # default features
//! [dependencies]
//! pubnub = "0.0.0"
//!
//! # all features
//! [dependencies]
//! pubnub = { version = "0.0.0", features = ["full"] }
//! ```
//!
//! ### Example
//!
//! Try the following sample code to get up and running quickly!
//!
//! ```no_run
//! use pubnub::{Keyset, PubNubClientBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let publish_key = "my_publish_key";
//!     let subscribe_key = "my_subscribe_key";
//!
//!     let client = PubNubClientBuilder::with_reqwest_transport()
//!         .with_keyset(Keyset {
//!             subscribe_key,
//!             publish_key: Some(publish_key),
//!             secret_key: None,
//!         })
//!         .with_user_id("user_id")
//!         .build()?;
//!
//!     client
//!         .publish_message("hello world!")
//!         .channel("my_channel")
//!         .r#type("text-message")
//!         .execute()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! You can find more examples in our [examples](examples/) directory!
//!
//! ## Features
//! The `pubnub` crate is split into multiple features, which can be enabled or disabled in your `Cargo.toml` file.
//! Feature list:
//! * `full` - enables all not conflicting features
//! * `serde` - uses [serde](https://github.com/serde-rs/serde) for serialization
//! * `reqwest` - uses [reqwest](https://github.com/seanmonstar/reqwest) as a transport layer
//! * `blocking` - enables blocking API
//! * `aescbc` - enables AES-CBC encryption
//! * `default` - default features that include:
//!    * `serde`
//!    * `reqwest`
//!    * `aescbc`
//!
//! ## Documentation
//!
//! :warning: We are under the development! **Links below not work** :warning:
//!
//! * [API reference for Rust](https://www.pubnub.com/docs/sdks/rust)
//! * [Rust docs](https://www.docs.rs/pubnub/latest/pubnub)
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

#[doc(inline)]
pub use dx::access;
#[doc(inline)]
pub use dx::publish;
#[doc(inline)]
pub use dx::{Keyset, PubNubClient, PubNubClientBuilder};
pub mod dx;

pub mod core;
pub mod providers;
pub mod transport;

/// A facade around all the std types that we use in the library.
/// It is used to make the library `no_std` compatible.
mod lib {
    #[cfg(not(feature = "std"))]
    extern crate alloc as std_alloc;

    pub(crate) mod core {
        #[cfg(not(feature = "std"))]
        pub(crate) use core::*;
        #[cfg(feature = "std")]
        pub(crate) use std::*;
    }

    pub(crate) mod alloc {
        #[cfg(not(feature = "std"))]
        pub(crate) use super::std_alloc::*;

        #[cfg(feature = "std")]
        pub(crate) use std::*;
    }

    pub(crate) mod collections {
        #[cfg(not(feature = "std"))]
        pub(crate) use super::alloc::collections::*;

        pub(crate) use hash_map::HashMap;

        pub(crate) mod hash_map {
            /// Depending of the `std` feature, this module will re-export
            /// either `std::collections::HashMap` or `hashbrown::HashMap`.
            /// This is needed because there is no `no_std` HashMap available.
            /// We decided to use `hashbrown` because it is fast and has same API as `std` HashMap.

            #[cfg(not(feature = "std"))]
            pub(crate) use hashbrown::HashMap;

            #[cfg(feature = "std")]
            pub(crate) use std::collections::HashMap;
        }
    }
}
