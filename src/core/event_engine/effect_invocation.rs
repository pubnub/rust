use crate::core::event_engine::Effect;

/// Effect invocation trait.
///
/// Invocation is an intention to run an effect. Effect dispatcher uses intents
/// to schedule actual effect invocation.
pub trait EffectInvocation {
    type Effect: Effect;

    /// Unique effect invocation identifier.
    fn id(&self) -> &str;

    /// Whether invoked effect lifetime should be managed by dispatcher or not.
    fn managed(&self) -> bool;

    /// Whether effect invocation cancels managed effect or not.
    fn cancelling(&self) -> bool;

    /// Whether effect invocation cancels specific managed effect or not.
    fn cancelling_effect(&self, effect: &Self::Effect) -> bool;
}
