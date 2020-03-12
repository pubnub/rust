use crate::runtime::Runtime;
use crate::subscription::subscribe_loop_supervisor::SubscribeLoopSupervisor;
use crate::transport::{Service, Transport};
use futures_util::lock::Mutex;
use std::sync::Arc;

mod presence;
mod publish;
mod subscribe;

#[cfg(test)]
mod tests;

/// # PubNub Client
///
/// The PubNub lib implements socket pools to relay data requests as a client
/// connection to the PubNub Network.
#[derive(Clone, Debug)]
pub struct PubNub<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
    /// Transport to use for communication.
    pub(crate) transport: TTransport,
    /// Runtime to use for managing resources.
    pub(crate) runtime: TRuntime,

    /// Subscribe loop lifecycle management.
    pub(crate) subscribe_loop_supervisor: Arc<Mutex<SubscribeLoopSupervisor>>,
}

impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
where
    TTransport: Transport + 'static,
    TRuntime: Runtime + 'static,
{
    /// Get a reference to a transport being used.
    pub fn transport(&self) -> &TTransport {
        &self.transport
    }

    /// Get a reference to a runtime being used.
    pub fn runtime(&self) -> &TRuntime {
        &self.runtime
    }
}

impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
where
    TTransport: Transport + 'static,
    TRuntime: Runtime + 'static,
{
    /// Perform a transport call.
    ///
    /// # Errors
    ///
    /// Returns transport-specific errors.
    pub async fn call<TRequest>(
        &self,
        req: TRequest,
    ) -> Result<<TTransport as Service<TRequest>>::Response, <TTransport as Service<TRequest>>::Error>
    where
        TTransport: Service<TRequest>,
    {
        self.transport.call(req).await
    }
}
