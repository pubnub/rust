use crate::dx::subscribe::event_engine::effect_handler::EmitData;
use crate::dx::subscribe::event_engine::{effect_handler::EmitFunction, SubscribeEvent};
use crate::dx::subscribe::SubscribeStatus;
use crate::lib::alloc::{vec, vec::Vec};

pub(super) fn execute(
    status: SubscribeStatus,
    executor: EmitFunction,
) -> Option<Vec<SubscribeEvent>> {
    executor(EmitData::SubscribeStatus(status))
        .map(|_| vec![])
        .ok()
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::PubNubError;

    #[test]
    fn emit_status() {
        fn mock_handshake_function(data: EmitData) -> Result<(), PubNubError> {
            assert!(matches!(data, EmitData::SubscribeStatus(_)));

            Ok(())
        }

        let result = execute(SubscribeStatus::Connected, mock_handshake_function);

        assert!(result.is_some());
        assert!(result.unwrap().is_empty())
    }

    #[test]
    fn return_emit_failure_event_on_err() {
        fn mock_handshake_function(_data: EmitData) -> Result<(), PubNubError> {
            Err(PubNubError::Transport {
                details: "test".into(),
            })
        }

        assert!(execute(SubscribeStatus::Connected, mock_handshake_function).is_none());
    }
}
