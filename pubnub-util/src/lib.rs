//! # Shared PubNub utilities.
//! May come in handy when implemeting custom transports.

#![deny(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    intra_doc_link_resolution_failure
)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

#[cfg(feature = "url-encoded-list")]
pub mod url_encoded_list;
