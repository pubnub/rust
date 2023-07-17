//! Event Engine module

use crate::lib::alloc::{sync::Arc, vec::Vec};
use async_channel::Sender;
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

use crate::core::runtime::Runtime;
#[doc(inline)]
pub(crate) use transition::Transition;

pub(crate) mod transition;

pub(crate) mod effect_execution;

/// State machine's event engine.
///
/// [`EventEngine`] is the core of state machines used in PubNub client and
/// manages current system state and handles external events.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct EventEngine<S, EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF> + Send + Sync,
    EH: EffectHandler<EI, EF>,
    EF: Effect<Invocation = EI>,
    S: State<State = S, Invocation = EI> + Send + Sync,
{
    /// Effects dispatcher.
    ///
    /// Dispatcher responsible for effects invocation processing.
    effect_dispatcher: Arc<EffectDispatcher<EH, EF, EI>>,

    /// `Effect invocation` submission channel.
    ///
    /// Channel is used to submit `invocations` for new effects execution.
    effect_dispatcher_channel: Sender<EI>,

    /// Current event engine state.
    current_state: RwLock<S>,
}

impl<S, EH, EF, EI> EventEngine<S, EH, EF, EI>
where
    S: State<State = S, Invocation = EI> + Send + Sync + 'static,
    EH: EffectHandler<EI, EF> + Send + Sync + 'static,
    EF: Effect<Invocation = EI> + 'static,
    EI: EffectInvocation<Effect = EF> + Send + Sync + 'static,
{
    /// Create [`EventEngine`] with initial state for state machine.
    #[allow(dead_code)]
    pub fn new<R>(handler: EH, state: S, runtime: R) -> Arc<Self>
    where
        R: Runtime,
    {
        let (channel_tx, channel_rx) = async_channel::bounded::<EI>(5);
        let effect_dispatcher = Arc::new(EffectDispatcher::new(handler, channel_rx));

        let engine = Arc::new(EventEngine {
            effect_dispatcher,
            effect_dispatcher_channel: channel_tx.clone(),
            current_state: RwLock::new(state),
        });

        engine.start(runtime);

        engine
    }

    /// Retrieve current engine state.
    #[allow(dead_code)]
    pub fn current_state(&self) -> S {
        (*self.current_state.read()).clone()
    }

    /// Process external event.
    ///
    /// Process event passed to the system and perform required transitions to
    /// new state if required.
    #[allow(dead_code)]
    pub fn process(&self, event: &EI::Event) {
        let state = self.current_state.read();

        let transition = state.transition(event);

        drop(state);

        transition.map(|transition| self.process_transition(transition));
    }

    /// Process transition.
    ///
    /// This method is responsible for transition maintenance:
    /// * update current state
    /// * call effects dispatcher to process effect invocation
    fn process_transition(&self, transition: Transition<S::State, S::Invocation>) {
        {
            let mut writable_state = self.current_state.write();
            *writable_state = transition.state;
        }

        transition.invocations.into_iter().for_each(|invocation| {
            self.effect_dispatcher_channel
                .send_blocking(invocation)
                .unwrap();
        });
    }

    /// Start state machine.
    ///
    /// This method is used to start state machine and perform initial State
    /// transition.
    fn start<R>(self: &Arc<Self>, runtime: R)
    where
        R: Runtime,
    {
        let engine_clone = self.clone();
        let dispatcher = self.effect_dispatcher.clone();

        dispatcher.start(
            move |events| {
                events.iter().for_each(|event| engine_clone.process(event));
            },
            runtime,
        );
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::lib::alloc::{vec, vec::Vec};
    use std::future::Future;

    #[derive(Debug, Clone, PartialEq)]
    enum TestState {
        NotStarted,
        Started,
        InProgress,
        Completed,
    }

    impl State for TestState {
        type State = Self;
        type Invocation = TestInvocation;
        type Event = TestEvent;

        fn enter(&self) -> Option<Vec<Self::Invocation>> {
            Some(vec![TestInvocation::One])
        }

        fn exit(&self) -> Option<Vec<Self::Invocation>> {
            Some(vec![TestInvocation::Two])
        }

        fn transition(
            &self,
            event: &<<Self as State>::Invocation as EffectInvocation>::Event,
        ) -> Option<Transition<Self::State, Self::Invocation>> {
            match event {
                TestEvent::One => {
                    if matches!(self, Self::NotStarted) {
                        Some(self.transition_to(Self::Started, None))
                    } else if matches!(self, Self::Completed) {
                        Some(
                            self.transition_to(Self::NotStarted, Some(vec![TestInvocation::Three])),
                        )
                    } else {
                        None
                    }
                }
                TestEvent::Two => matches!(self, Self::Started)
                    .then(|| self.transition_to(Self::InProgress, None)),
                TestEvent::Three => matches!(self, Self::InProgress)
                    .then(|| self.transition_to(Self::Completed, Some(vec![TestInvocation::One]))),
            }
        }

        fn transition_to(
            &self,
            state: Self::State,
            invocations: Option<Vec<Self::Invocation>>,
        ) -> Transition<Self::State, Self::Invocation> {
            Transition {
                invocations: self
                    .exit()
                    .unwrap_or(vec![])
                    .into_iter()
                    .chain(invocations.unwrap_or(vec![]).into_iter())
                    .chain(state.enter().unwrap_or(vec![]).into_iter())
                    .collect(),
                state,
            }
        }
    }

    enum TestEvent {
        One,
        Two,
        Three,
    }

    impl Event for TestEvent {
        fn id(&self) -> &str {
            match self {
                TestEvent::One => "EVENT_ONE",
                TestEvent::Two => "EVENT_TWO",
                TestEvent::Three => "EVENT_THREE",
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum TestEffect {
        One,
        Two,
        Three,
    }

    impl Effect for TestEffect {
        type Invocation = TestInvocation;

        fn id(&self) -> String {
            match self {
                Self::One => "EFFECT_ONE".into(),
                Self::Two => "EFFECT_TWO".into(),
                Self::Three => "EFFECT_THREE".into(),
            }
        }

        fn run<F>(&self, _: F) {
            // Do nothing.
        }

        fn cancel(&self) {
            // Do nothing.
        }
    }

    enum TestInvocation {
        One,
        Two,
        Three,
    }

    impl EffectInvocation for TestInvocation {
        type Effect = TestEffect;
        type Event = TestEvent;

        fn id(&self) -> &str {
            match self {
                Self::One => "EFFECT_ONE_INVOCATION",
                Self::Two => "EFFECT_TWO_INVOCATION",
                Self::Three => "EFFECT_THREE_INVOCATION",
            }
        }

        fn managed(&self) -> bool {
            matches!(self, Self::Two | Self::Three)
        }

        fn cancelling(&self) -> bool {
            false
        }

        fn cancelling_effect(&self, _effect: &Self::Effect) -> bool {
            false
        }
    }

    struct TestEffectHandler {}

    impl EffectHandler<TestInvocation, TestEffect> for TestEffectHandler {
        fn create(&self, invocation: &TestInvocation) -> Option<TestEffect> {
            match invocation {
                TestInvocation::One => Some(TestEffect::One),
                TestInvocation::Two => Some(TestEffect::Two),
                TestInvocation::Three => Some(TestEffect::Three),
            }
        }
    }

    #[derive(Clone)]
    struct TestRuntime {}

    impl Runtime for TestRuntime {
        fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static)
        where
            R: Send + 'static,
        {
            // Do nothing.
        }
    }

    #[test]
    fn set_initial_state() {
        let engine = EventEngine::new(TestEffectHandler {}, TestState::NotStarted, TestRuntime {});
        assert!(matches!(engine.current_state(), TestState::NotStarted));
    }

    #[test]
    fn transit_to_new_state() {
        let engine = EventEngine::new(TestEffectHandler {}, TestState::NotStarted, TestRuntime {});
        engine.process(&TestEvent::One);
        assert!(matches!(engine.current_state(), TestState::Started));
    }

    #[test]
    fn transit_between_states() {
        let engine = EventEngine::new(TestEffectHandler {}, TestState::NotStarted, TestRuntime {});

        engine.process(&TestEvent::One);
        assert!(matches!(engine.current_state(), TestState::Started));

        engine.process(&TestEvent::Two);
        assert!(matches!(engine.current_state(), TestState::InProgress));

        engine.process(&TestEvent::Three);
        assert!(matches!(*engine.current_state.read(), TestState::Completed));

        engine.process(&TestEvent::One);
        assert!(matches!(
            *engine.current_state.read(),
            TestState::NotStarted
        ));
    }

    #[test]
    fn not_transit_for_unexpected_event() {
        let engine = EventEngine::new(TestEffectHandler {}, TestState::NotStarted, TestRuntime {});

        engine.process(&TestEvent::One);
        assert!(matches!(engine.current_state(), TestState::Started));

        engine.process(&TestEvent::Three);
        assert!(!matches!(engine.current_state(), TestState::Completed));
        assert!(matches!(engine.current_state(), TestState::Started));
    }

    #[test]
    fn return_effects_to_process() {
        let engine = EventEngine::new(TestEffectHandler {}, TestState::NotStarted, TestRuntime {});

        let effects = engine.process(&TestEvent::One);

        assert_eq!(effects, vec![Arc::new(TestEffect::One)]);
    }
}
