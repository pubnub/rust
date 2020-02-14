#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

#[cfg(feature = "encoded-channels-list")]
pub mod encoded_channels_list;
