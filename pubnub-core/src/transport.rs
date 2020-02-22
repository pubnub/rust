use crate::data::{presence, request, response};
use async_trait::async_trait;

/// Transport abstracts away the underlying mechanism through which the PubNub
/// client communicates with the PubNub network.
pub trait Transport:
    Clone
    + Send
    + Sync
    // Publish.
    + Service<request::Publish, Response = response::Publish, Error = <Self as Transport>::Error>
    // Subscribe.
    + Service<request::Subscribe, Response = response::Subscribe, Error = <Self as Transport>::Error>
    // Set state.
    + Service<request::SetState, Response = response::SetState, Error = <Self as Transport>::Error>
    // Get state.
    + Service<request::GetState, Response = response::GetState, Error = <Self as Transport>::Error>
    // Here now.
    + Service<request::HereNow<presence::respond_with::OccupancyOnly>, Response = response::HereNow<presence::respond_with::OccupancyOnly>, Error = <Self as Transport>::Error>
    + Service<request::HereNow<presence::respond_with::OccupancyAndUUIDs>, Response = response::HereNow<presence::respond_with::OccupancyAndUUIDs>, Error = <Self as Transport>::Error>
    + Service<request::HereNow<presence::respond_with::Full>, Response = response::HereNow<presence::respond_with::Full>, Error = <Self as Transport>::Error>
    // Global Here now.
    + Service<request::GlobalHereNow<presence::respond_with::OccupancyOnly>, Response = response::GlobalHereNow<presence::respond_with::OccupancyOnly>, Error = <Self as Transport>::Error>
    + Service<request::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>, Response = response::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>, Error = <Self as Transport>::Error>
    + Service<request::GlobalHereNow<presence::respond_with::Full>, Response = response::GlobalHereNow<presence::respond_with::Full>, Error = <Self as Transport>::Error>
    // Where now.
    + Service<request::WhereNow, Response = response::WhereNow, Error = <Self as Transport>::Error>
{
    /// Transport-specific error type this transport can generate.
    type Error: std::error::Error + Send + Sync;
}

/// Service respresents a single unit of an async request/response based API.
#[async_trait]
pub trait Service<Request>: Send {
    /// Response given by the service.
    type Response;
    /// Error produced by the service.
    type Error;

    /// Process the request and return the response asynchronously.
    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error>;
}
