use super::encoded_channels_list::EncodedChannelsList;
use crate::transport::Transport;
use crate::{Message, Timetoken};
use log::debug;

// TODO: change the Trasport layer to directly work with this kind of type.
#[allow(clippy::module_name_repetitions)]
pub(crate) struct SubscribeRequestParams<'a> {
    pub origin: &'a str,
    pub subscribe_key: &'a str,
    pub encoded_channels: &'a EncodedChannelsList,
}

pub(crate) async fn subscribe_request<T: Transport>(
    transport: &T,
    params: SubscribeRequestParams<'_>,
    timetoken: Timetoken,
) -> Result<(Vec<Message>, Timetoken), T::Error> {
    // Construct URI
    // TODO:
    // - auth key
    // - uuid
    // - signatures
    // - channel groups
    // - filters
    let url = format!(
        "https://{origin}/v2/subscribe/{sub_key}/{channels}/0?tt={tt}&tr={tr}",
        origin = params.origin,
        sub_key = params.subscribe_key,
        channels = params.encoded_channels,
        tt = timetoken.t,
        tr = timetoken.r,
    );
    debug!("URL: {}", url);

    // Send network request
    let url = url.parse().expect("Unable to parse URL");
    transport.subscribe_request(url).await
}
