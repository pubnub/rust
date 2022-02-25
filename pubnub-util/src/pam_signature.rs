//! PAM v3 signature implemetnation.

use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

/// The request to sign.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    /// PubNub publish key.
    pub publish_key: &'a str,
    /// Request method.
    pub method: &'a str,
    /// Path component of the URL.
    pub path: &'a str,
    /// Query component of the URL.
    pub query: &'a str,
    /// Request body.
    pub body: &'a str,
}

/// Sign a request using the specified parameters.
#[must_use]
pub fn sign(secret: &str, request: Request<'_>) -> String {
    let plain_message = format_plain_message(request);
    let encrypted_message = encrypt(secret, &plain_message);
    let base64_encrypted_message =
        base64::encode_config(encrypted_message, base64::URL_SAFE_NO_PAD);
    format!("v2.{}", base64_encrypted_message)
}

fn format_plain_message(request: Request<'_>) -> String {
    format!(
        "{method}\n{pub_key}\n{path}\n{query_string}\n{body}",
        method = request.method,
        pub_key = request.publish_key,
        path = request.path,
        query_string = request.query,
        body = request.body,
    )
}

type HmacSha256 = Hmac<Sha256>;

fn encrypt(secret: &str, plain_message: &str) -> [u8; 32] {
    let mut mac = HmacSha256::new_varkey(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(plain_message.as_bytes());
    let code = mac.finalize().into_bytes();
    code.into()
}

#[cfg(test)]
mod tests {
    use super::{format_plain_message, sign, Request};

    const TEST_BODY_BLOB: &str = include_str!("../testdata/pam_signature/body.json");
    const EXPECTED_FORMATTED_BLOB: &str = include_str!("../testdata/pam_signature/formatted.blob");
    const SAMPLE_REQUEST: Request<'_> = Request {
        publish_key: "demo",
        method: "POST",
        path: "/v3/pam/demo/grant",
        query: "PoundsSterling=%C2%A313.37&timestamp=123456789",
        body: TEST_BODY_BLOB,
    };

    #[test]
    fn test_format_plain_message() {
        assert_eq!(
            format_plain_message(SAMPLE_REQUEST),
            EXPECTED_FORMATTED_BLOB,
        );
    }

    #[test]
    fn test_sign() {
        assert_eq!(
            sign("wMfbo9G0xVUG8yfTfYw5qIdfJkTd7A", SAMPLE_REQUEST),
            "v2.W7Vim_epW4RyuT427E7cS2HiMADRP0wLP6-RkTWPtaM",
        );
    }
}
