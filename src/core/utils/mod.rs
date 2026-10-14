#[cfg(any(
    feature = "publish",
    feature = "access",
    feature = "subscribe",
    feature = "presence"
))]
pub mod encoding;
#[cfg(any(
    feature = "publish",
    feature = "access",
    feature = "subscribe",
    feature = "presence"
))]
pub mod headers;

pub mod metadata;

#[cfg(any(feature = "access", feature = "parse_token"))]
pub(crate) mod permission_bits;
