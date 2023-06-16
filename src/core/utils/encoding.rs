use crate::lib::alloc::{
    string::{String, ToString},
    vec::Vec,
};
use percent_encoding::{percent_encode, AsciiSet, CONTROLS};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// https://url.spec.whatwg.org/#path-percent-encode-set
const PATH: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');

/// https://url.spec.whatwg.org/#userinfo-percent-encode-set
const USERINFO: &AsciiSet = &PATH
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

/// `+` sign needed by PubNub API
const PUBNUB_SET: &AsciiSet = &USERINFO.add(b'+').add(b'%');

/// `percent_encoding` crate recommends you to create your own set for encoding.
/// To be consistent in the whole codebase - we created a function that can be used
/// for encoding related stuff.
pub fn url_encode(data: &[u8]) -> String {
    percent_encode(data, PUBNUB_SET).to_string()
}

/// Join list of encoded strings.
pub fn join_url_encoded(strings: &[&str], sep: &str) -> Option<String> {
    if strings.is_empty() {
        return None;
    }

    Some(
        strings
            .iter()
            .map(|val| url_encode(val.as_bytes()))
            .collect::<Vec<String>>()
            .join(sep),
    )
}
