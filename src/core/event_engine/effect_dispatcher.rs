use crate::{
    core::event_engine::{Effect, EffectHandler, EffectInvocation},
    lib::alloc::{rc::Rc, vec, vec::Vec},
};
use phantom_type::PhantomType;
use spin::rwlock::RwLock;

/// State machine effects dispatcher.
#[allow(dead_code)]
pub(crate) struct EffectDispatcher<EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF>,
    EH: EffectHandler<EI, EF>,
    EF: Effect<Invocation = EI>,
{
    /// Effect invocation handler.
    ///
    /// Handler responsible for providing actual implementation of
    handler: EH,

    /// Dispatched effects managed by dispatcher.
    ///
    /// There are effects whose lifetime should be managed by the dispatcher.
    /// State machines may have some effects that are exclusive and can only run
    /// one type of them at once. The dispatcher handles such effects
    /// and cancels them when required.
    managed: RwLock<Vec<Rc<EF>>>,

    _invocation: PhantomType<EI>,
}

impl<EH, EF, EI> EffectDispatcher<EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF>,
    EH: EffectHandler<EI, EF>,
    EF: Effect<Invocation = EI>,
{
    /// Create new effects dispatcher.
    pub fn new(handler: EH) -> Self {
        EffectDispatcher {
            handler,
            managed: RwLock::new(vec![]),
            _invocation: Default::default(),
        }
    }

    /// Dispatch effect associated with `invocation`.
    pub fn dispatch<F>(&self, invocation: &EI, mut f: F)
    where
        F: FnMut(Option<Vec<EI::Event>>),
    {
        if let Some(effect) = self.handler.create(invocation) {
            let effect = Rc::new(effect);

            if invocation.managed() {
                let mut managed = self.managed.write();
                managed.push(effect.clone());
            }

            // Placeholder for effect invocation.
            effect.run(|events| {
                // Try remove effect from list of managed.
                self.remove_managed_effect(&effect);

                // Notify about effect run completion.
                // Placeholder for effect events processing (pass to effects handler).
                f(events);
            });
        } else if invocation.cancelling() {
            self.cancel_effect(invocation);

            // Placeholder for effect events processing (pass to effects handler).
            f(None);
        }
    }

    /// Handle effect cancellation.
    ///
    /// Effects with managed lifecycle can be cancelled by corresponding effect
    /// invocations.
    fn cancel_effect(&self, invocation: &EI) {
        let mut managed = self.managed.write();
        if let Some(position) = managed.iter().position(|e| invocation.cancelling_effect(e)) {
            managed.remove(position).cancel();
        }
    }

    /// Remove managed effect.
    fn remove_managed_effect(&self, effect: &EF) {
        let mut managed = self.managed.write();
        if let Some(position) = managed.iter().position(|ef| ef.id() == effect.id()) {
            managed.remove(position);
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::event_engine::Event;

    enum TestEvent {}

    impl Event for TestEvent {
        fn id(&self) -> &str {
            "no_id"
        }
    }

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

        fn run<F>(&self, mut f: F)
        where
            F: FnMut(Option<Vec<TestEvent>>),
        {
            match self {
                Self::Three => {}
                _ => f(None),
            }
        }

        fn cancel(&self) {
            // Do nothing.
        }
    }

    enum TestInvocation {
        One,
        Two,
        Three,
        CancelThree,
    }

    impl EffectInvocation for TestInvocation {
        type Effect = TestEffect;
        type Event = TestEvent;

        fn id(&self) -> &str {
            match self {
                Self::One => "EFFECT_ONE_INVOCATION",
                Self::Two => "EFFECT_TWO_INVOCATION",
                Self::Three => "EFFECT_THREE_INVOCATION",
                Self::CancelThree => "EFFECT_THREE_CANCEL_INVOCATION",
            }
        }

        fn managed(&self) -> bool {
            matches!(self, Self::Two | Self::Three)
        }

        fn cancelling(&self) -> bool {
            matches!(self, Self::CancelThree)
        }

        fn cancelling_effect(&self, effect: &Self::Effect) -> bool {
            match self {
                TestInvocation::CancelThree => matches!(effect, TestEffect::Three),
                _ => false,
            }
        }
    }

    struct TestEffectHandler {}

    impl EffectHandler<TestInvocation, TestEffect> for TestEffectHandler {
        fn create(&self, invocation: &TestInvocation) -> Option<TestEffect> {
            match invocation {
                TestInvocation::One => Some(TestEffect::One),
                TestInvocation::Two => Some(TestEffect::Two),
                TestInvocation::Three => Some(TestEffect::Three),
                _ => None,
            }
        }
    }

    #[test]
    fn run_not_managed_effect() {
        let mut called = false;
        let dispatcher = EffectDispatcher::new(TestEffectHandler {});
        dispatcher.dispatch(&TestInvocation::One, |_| {
            called = true;
        });

        assert!(called, "Expected to call effect for TestInvocation::One");
        assert_eq!(
            dispatcher.managed.read().len(),
            0,
            "Non managed effects shouldn't be stored"
        );
    }

    #[test]
    fn run_managed_effect() {
        let mut called = false;
        let dispatcher = EffectDispatcher::new(TestEffectHandler {});
        dispatcher.dispatch(&TestInvocation::Two, |_| {
            called = true;
        });

        assert!(called, "Expected to call effect for TestInvocation::Two");
        assert_eq!(
            dispatcher.managed.read().len(),
            0,
            "Managed effect should be removed on completion"
        );
    }

    #[test]
    fn cancel_managed_effect() {
        let mut called_managed = false;
        let mut cancelled_managed = false;
        let dispatcher = EffectDispatcher::new(TestEffectHandler {});
        dispatcher.dispatch(&TestInvocation::Three, |_| {
            called_managed = true;
        });

        assert!(
            !called_managed,
            "Expected that effect for TestInvocation::Three won't be called"
        );
        assert_eq!(
            dispatcher.managed.read().len(),
            1,
            "Managed effect shouldn't complete run because doesn't have completion call"
        );

        dispatcher.dispatch(&TestInvocation::CancelThree, |_| {
            cancelled_managed = true;
        });

        assert!(
            cancelled_managed,
            "Expected to call effect for TestInvocation::CancelThree"
        );
        assert!(
            !called_managed,
            "Expected that effect for TestInvocation::Three won't be called"
        );
        assert_eq!(
            dispatcher.managed.read().len(),
            0,
            "Managed effect should be cancelled"
        );
    }
}
