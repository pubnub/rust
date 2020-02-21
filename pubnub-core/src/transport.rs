use crate::data::{request, response};
use async_trait::async_trait;

/// Transport abstracts away the underlying mechanism through which the PubNub
/// client communicates with the PubNub network.
pub trait Transport:
    Clone
    + Send
    + Sync
    // Send a Publish Request and return the timetoken.
    + Service<request::Publish, Response = response::Publish, Error = <Self as Transport>::Error>
    // Send a Subscribe Request and return the messages received.
    + Service<request::Subscribe, Response = response::Subscribe, Error = <Self as Transport>::Error>
    // Send a Set State Request and return the response.
    + Service<request::SetState, Response = response::SetState, Error = <Self as Transport>::Error>
    // Send a Get State Request and return the state.
    + Service<request::GetState, Response = response::GetState, Error = <Self as Transport>::Error>
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
