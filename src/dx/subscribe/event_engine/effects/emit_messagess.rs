use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{effects::EmitMessagesEffectExecutor, SubscribeEvent},
        result::Update,
    },
    lib::alloc::sync::Arc,
};
use futures::{future::BoxFuture, FutureExt};
use log::info;

pub(super) fn execute(
    updates: Vec<Update>,
    executor: &Arc<EmitMessagesEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!("Emit updates: {updates:?}");

    let cloned_executor = executor.clone();

    //    async move {
    //        cloned_executor(updates);
    //
    //        Ok(vec![])
    //    }
    //    .boxed()
    vec![]
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::dx::subscribe::types::Message;

    #[tokio::test]
    async fn emit_expected_status() {
        let message = Message {
            sender: Some("test-user".into()),
            timestamp: 1234567890,
            channel: "test".to_string(),
            subscription: "test-group".to_string(),
            data: vec![],
            r#type: None,
            space_id: None,
            decryption_error: None,
        };
        let mut function_called = false;
        let emit_message_function: Arc<EmitMessagesEffectExecutor> = Arc::new(|updates| {
            let emitted_update = updates.first().expect("update should be passed");
            assert!(matches!(emitted_update, Update::Message(_)));

            if let Update::Message(message) = emitted_update {
                assert_eq!(*message, message.clone());
            }

            function_called = true;
        });

        execute(
            vec![Update::Message(message.clone())],
            &emit_message_function,
        )
        .await
        .expect("expected result");

        assert!(function_called);
    }
}
