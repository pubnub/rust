use async_channel::Receiver;

use crate::core::PubNubError;

/// TODO: why is it public needed for request?
#[derive(Debug)]
pub struct CancellationTask {
    cancel_rx: Receiver<String>,
    id: String,
}

impl CancellationTask {
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
                    details: format!("Cancellation pipe failed: {err}"),
                })?
                .eq(&self.id)
            {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[tokio::test]
    async fn wait_for_cancel() {
        let (cancel_tx, cancel_rx) = async_channel::bounded(2);

        let cancel_task = CancellationTask::new(cancel_rx, "id".into());

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            cancel_tx.send("id".into()).await.unwrap();
        });

        cancel_task.wait_for_cancel().await.unwrap();
    }
}