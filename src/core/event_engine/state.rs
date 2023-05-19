use crate::{
    core::event_engine::{EffectInvocation, Event, Transition},
    lib::alloc::vec::Vec,
};

/// State machine state trait.
///
/// For transition, the state machine needs to know which effects should be
/// dispatched during transition to target state in response to a specific
/// event.
///
/// Types which are expected to be used as states should implement the trait.
pub(crate) trait State {
    type State: State;
    type Invocation: EffectInvocation;
    type Event: Event;

    /// State enter effects invocations.
    ///
    /// The list of effect invocations that should be called when the event
    /// engine enters state.
    fn enter(&self) -> Option<Vec<Self::Invocation>>;

    /// State exit effects invocations.
    ///
    /// The list of effect invocations that should be called when the event
    /// engine leaves state.
    fn exit(&self) -> Option<Vec<Self::Invocation>>;

    /// System event handler.
    ///
    /// State has information about the next state into which the state machine
    /// should switch and a list of effects invocations which should be
    /// scheduled.
    fn transition(&self, event: &Self::Event) -> Option<Transition<Self::State, Self::Invocation>>;

    /// [`Transition`] build helper.
    ///
    /// Transition to a new state is a composite operation which includes
    /// dispatching of [`exit`] effect invocations of receiver, followed by
    /// dispatch of provided transition effect `invocations` and [`enter`]
    /// effect invocations of target state.
    fn transition_to(
        &self,
        state: Self::State,
        invocations: Option<Vec<Self::Invocation>>,
    ) -> Transition<Self::State, Self::Invocation>;
}
