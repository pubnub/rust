//! Macros.

/// Encodes JSON into a urlencoded coding.
#[macro_export]
macro_rules! encode_json {
    ($value:expr => $to:ident) => {
        let value_string = ::pubnub_core::json::stringify($value);
        let $to = {
            use ::percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
            utf8_percent_encode(&value_string, NON_ALPHANUMERIC)
        };
    };
}
