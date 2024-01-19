use crate::core::event_engine::{Effect, Event};

/// Effect invocation trait.
///
/// Invocation is an intention to run an effect. Effect dispatcher uses intents
/// to schedule actual effect invocation.
pub(crate) trait EffectInvocation {
    type Effect: Effect;
    type Event: Event;

    /// Unique effect invocation identifier.
    fn id(&self) -> &str;

    /// Whether invoked effect lifetime should be managed by dispatcher or not.
    fn is_managed(&self) -> bool;

    /// Whether effect invocation cancels managed effect or not.
    fn is_cancelling(&self) -> bool;

    /// Whether effect invocation cancels specific managed effect or not.
    fn cancelling_effect(&self, effect: &Self::Effect) -> bool;

    /// Whether invoked effect invocation should terminate current Event Engine
    /// processing loop or not.
    fn is_terminating(&self) -> bool;
}
