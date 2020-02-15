use crate::pubnub::PubNub;
use crate::runtime::Runtime;
use crate::subscription::subscribe_loop::ExitTx as SubscribeLoopExitTx;
use crate::subscription::subscribe_loop_supervisor::{
    SubscribeLoopSupervisor, SubscribeLoopSupervisorParams,
};
use crate::transport::Transport;
use futures_util::lock::Mutex;
use std::sync::Arc;

/// # PubNub Client Builder
///
/// Create a [`crate::PubNub`] client using the builder pattern.
/// Optional items can be overridden using this.
#[derive(Clone, Debug)]
pub struct Builder<TTransport = (), TRuntime = ()> {
    /// Transport to use for communication.
    transport: TTransport,
    /// Runtime to use for managing resources.
    runtime: TRuntime,

    /// Subscription related configuration params.
    /// If set, gets a signal when subscribe loop exits.
    subscribe_loop_exit_tx: Option<SubscribeLoopExitTx>,
}

impl<TTransport, TRuntime> Builder<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
    /// Build the [`PubNub`] client to begin streaming messages.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_core::mock::{runtime::MockRuntime, transport::MockTransport};
    /// use pubnub_core::Builder;
    ///
    /// let transport = MockTransport::new();
    /// let runtime = MockRuntime::new();
    ///
    /// let pubnub = Builder::with_components(transport, runtime).build();
    /// ```
    #[must_use]
    pub fn build(self) -> PubNub<TTransport, TRuntime> {
        let Self {
            transport,
            runtime,
            subscribe_loop_exit_tx,
        } = self;

        let subscribe_loop_supervisor_params = SubscribeLoopSupervisorParams {
            exit_tx: subscribe_loop_exit_tx,
        };

        PubNub {
            transport,
            runtime,

            subscribe_loop_supervisor: Arc::new(Mutex::new(SubscribeLoopSupervisor::new(
                subscribe_loop_supervisor_params,
            ))),
        }
    }
}

impl<TTransport, TRuntime> Builder<TTransport, TRuntime> {
    /// Create a new [`Builder`] that can configure a [`PubNub`] client
    /// with custom component implementations.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_core::mock::{runtime::MockRuntime, transport::MockTransport};
    /// use pubnub_core::Builder;
    ///
    /// let transport = MockTransport::new();
    /// let runtime = MockRuntime::new();
    ///
    /// let pubnub = Builder::with_components(transport, runtime).build();
    /// ```
    #[must_use]
    pub fn with_components(transport: TTransport, runtime: TRuntime) -> Self {
        Self {
            subscribe_loop_exit_tx: None,

            transport,
            runtime,
        }
    }

    /// Set the subscribe loop exit tx.
    ///
    /// If set, subscribe loop sends a message to it when it exits.
    ///
    /// # Example
    ///
    /// ```
    /// # use pubnub_core::mock::{transport::MockTransport, runtime::MockRuntime};
    /// # let transport = MockTransport::new();
    /// # let runtime = MockRuntime::new();
    /// use pubnub_core::Builder;
    ///
    /// let (tx, _rx) = futures_channel::mpsc::channel(1);
    ///
    /// let pubnub = Builder::with_components(transport, runtime)
    ///     .subscribe_loop_exit_tx(tx)
    ///     .build();
    /// ```
    #[must_use]
    pub fn subscribe_loop_exit_tx(mut self, tx: SubscribeLoopExitTx) -> Self {
        self.subscribe_loop_exit_tx = Some(tx);
        self
    }

    /// Set the transport to use.
    ///
    /// This allows changing the [`Transport`] type on the builder and,
    /// therefore, on the resulting [`PubNub`] client.
    #[must_use]
    pub fn transport<U: Transport>(self, transport: U) -> Builder<U, TRuntime> {
        Builder {
            transport,

            // Copy the rest of the fields.
            runtime: self.runtime,
            subscribe_loop_exit_tx: self.subscribe_loop_exit_tx,
        }
    }

    /// Set the runtime to use.
    ///
    /// This allows changing the [`Runtime`] type on the builder and,
    /// therefore, on the resulting [`PubNub`] client.
    #[must_use]
    pub fn runtime<U: Runtime>(self, runtime: U) -> Builder<TTransport, U> {
        Builder {
            runtime,

            // Copy the rest of the fields.
            transport: self.transport,
            subscribe_loop_exit_tx: self.subscribe_loop_exit_tx,
        }
    }
}

impl Builder<(), ()> {
    /// Create a new [`Builder`] that can configure a [`PubNub`] client.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_core::mock::{runtime::MockRuntime, transport::MockTransport};
    /// use pubnub_core::Builder;
    ///
    /// let transport = MockTransport::new();
    /// let runtime = MockRuntime::new();
    ///
    /// let pubnub = Builder::new().transport(transport).runtime(runtime).build();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_components((), ())
    }
}

impl<TTransport, TRuntime> Default for Builder<TTransport, TRuntime>
where
    TTransport: Default,
    TRuntime: Default,
{
    /// Create a new [`Builder`] that can configure a [`PubNub`] client
    /// with default components.
    #[must_use]
    fn default() -> Self {
        Self::with_components(TTransport::default(), TRuntime::default())
    }
}
