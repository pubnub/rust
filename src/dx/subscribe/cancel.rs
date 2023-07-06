use async_channel::Receiver;

use crate::core::PubNubError;

/// TODO: why is it public needed for request?
#[derive(Debug)]
pub struct CancelationTask {
    cancel_rx: Receiver<String>,
    id: String,
}

impl CancelationTask {
    pub(super) fn new(cancel_rx: Receiver<String>, id: String) -> Self {
        Self { cancel_rx, id }
    }

    pub async fn wait_for_cancel(&self) -> Result<(), PubNubError> {
        loop {
            if self
                .cancel_rx
                .recv()
                .await
                .map_err(|err| PubNubError::Transport {
                    details: format!("Cancelation pipe failed: {err}"),
                })?
                .eq(&self.id)
            {
                break;
            }
        }

        Ok(())
    }
}
