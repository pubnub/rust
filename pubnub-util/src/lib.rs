//! # Shared PubNub utilities.
//! May come in handy when implemeting custom transports.

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]

#[cfg(feature = "encoded-channels-list")]
pub mod encoded_channels_list;
