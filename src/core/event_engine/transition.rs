use crate::{
    core::event_engine::{EffectInvocation, State},
    lib::alloc::vec::Vec,
};

/// State machine transition type.
///
/// State transition with information about target state and list of effect
/// invocations.

pub(crate) struct Transition<S, I>
where
    S: State,
    I: EffectInvocation,
{
    /// Target state machine state.
    pub state: Option<S>,

    /// List of effect invocation which should be scheduled during transition.
    pub invocations: Vec<I>,
}
