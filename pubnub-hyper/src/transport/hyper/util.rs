#[macro_export]
macro_rules! encode_json {
    ($value:expr => $to:ident) => {
        let value_string = json::stringify($value);
        let $to = {
            use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
            utf8_percent_encode(&value_string, NON_ALPHANUMERIC)
        };
    };
}

pub async fn handle_json_response(
    response: Response<Body>,
) -> Result<json::JsonValue, error::Error> {
    let mut body = response.into_body();
    let mut bytes = Vec::new();

    // Receive the response as a byte stream
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk?);
    }

    // Convert the resolved byte stream to JSON.
    let data = std::str::from_utf8(&bytes)?;
    let data_json = json::parse(data)?;

    Ok(data_json)
}
