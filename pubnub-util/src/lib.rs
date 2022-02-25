//! # Shared PubNub utilities.
//! May come in handy when implemeting custom transports.

#![deny(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::doc_markdown)]
#![allow(unknown_lints)]
#![forbid(unsafe_code)]

#[cfg(feature = "url-encoded-list")]
pub mod url_encoded_list;

#[cfg(feature = "uritemplate_api")]
pub mod uritemplate;

#[cfg(feature = "pam_signature")]
pub mod pam_signature;
