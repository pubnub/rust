use log::info;

use crate::{
    dx::subscribe::{
        event_engine::{effects::EmitMessagesEffectExecutor, SubscribeEvent},
        result::Update,
        SubscriptionCursor,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};

pub(super) async fn execute(
    cursor: SubscriptionCursor,
    updates: Vec<Update>,
    executor: &Arc<EmitMessagesEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!("Emit updates: {updates:?}");

    executor(updates, cursor);

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
            user_metadata: None,
            decryption_error: None,
        };

        let emit_message_function: Arc<EmitMessagesEffectExecutor> = Arc::new(|updates, _| {
            let emitted_update = updates.first().expect("update should be passed");
            assert!(matches!(emitted_update, Update::Message(_)));

            if let Update::Message(message) = emitted_update {
                assert_eq!(*message, message.clone());
            }
        });

        execute(
            Default::default(),
            vec![Update::Message(message.clone())],
            &emit_message_function,
        )
        .await;
    }
}
