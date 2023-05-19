//! Event Engine module

use spin::rwlock::RwLock;

#[doc(inline)]
pub(crate) use effect::Effect;
pub(crate) mod effect;

#[doc(inline)]
pub(crate) use effect_dispatcher::EffectDispatcher;
pub(crate) mod effect_dispatcher;

#[doc(inline)]
pub(crate) use effect_handler::EffectHandler;
pub(crate) mod effect_handler;

#[doc(inline)]
pub(crate) use effect_invocation::EffectInvocation;
pub(crate) mod effect_invocation;

#[doc(inline)]
pub(crate) use event::Event;
pub(crate) mod event;

#[doc(inline)]
pub(crate) use state::State;
pub(crate) mod state;

#[doc(inline)]
pub(crate) use transition::Transition;
pub(crate) mod transition;

/// State machine's event engine.
///
/// [`EventEngine`] is the core of state machines used in PubNub client and
/// manages current system state and handles external events.
#[allow(dead_code)]
pub(crate) struct EventEngine<S, EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF>,
    EH: EffectHandler<EI, EF>,
    EF: Effect,
    S: State<State = S, Invocation = EI>,
{
    /// Effects dispatcher.
    ///
    /// Dispatcher responsible for effects invocation processing.
    effect_dispatcher: EffectDispatcher<EH, EF, EI>,

    /// Current event engine state.
    current_state: RwLock<S>,
}

impl<S, EH, EF, EI> EventEngine<S, EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF>,
    EH: EffectHandler<EI, EF>,
    EF: Effect,
    S: State<State = S, Invocation = EI>,
{
    /// Create [`EventEngine`] with initial state for state machine.
    #[allow(dead_code)]
    pub fn new(handler: EH, state: S) -> Self {
        EventEngine {
            effect_dispatcher: EffectDispatcher::new(handler),
            current_state: RwLock::new(state),
        }
    }

    /// Process external event.
    ///
    /// Process event passed to the system and perform required transitions to
    /// new state if required.
    #[allow(dead_code)]
    pub fn process(&self, event: &S::Event) {
        let state = self.current_state.read();
        if let Some(transition) = state.transition(event) {
            drop(state);
            self.process_transition(transition);
        }
    }

    /// Process transition.
    ///
    /// This method is responsible for transition maintenance:
    /// * update current state
    /// * call effects dispatcher to process effect invocation
    fn process_transition(&self, transition: Transition<S::State, S::Invocation>) {
        let mut writable_state = self.current_state.write();
        *writable_state = transition.state;
        drop(writable_state);

        transition.invocations.iter().for_each(|invocation| {
            self.effect_dispatcher.dispatch(invocation, |events| {
                events.iter().for_each(|event| self.process(event));
            });
        });
    }
}
