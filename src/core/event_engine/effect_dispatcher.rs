use crate::{
    core::event_engine::{Effect, EffectHandler, EffectInvocation},
    lib::alloc::{string::String, sync::Arc, vec, vec::Vec},
};
use phantom_type::PhantomType;
use spin::rwlock::RwLock;

/// State machine effects dispatcher.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct EffectDispatcher<EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF> + Send + Sync,
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
    managed: Arc<RwLock<Vec<Arc<EF>>>>,

    _invocation: PhantomType<EI>,
}

impl<EH, EF, EI> EffectDispatcher<EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF> + Send + Sync,
    EH: EffectHandler<EI, EF> + Send + Sync,
    EF: Effect<Invocation = EI> + 'static,
{
    /// Create new effects dispatcher.
    pub fn new(handler: EH) -> Self {
        EffectDispatcher {
            handler,
            managed: Arc::new(RwLock::new(vec![])),
            _invocation: Default::default(),
        }
    }

    /// Dispatch effect associated with `invocation`.
    pub fn dispatch(self: &Arc<Self>, invocation: &EI) -> Option<Arc<EF>> {
        if let Some(effect) = self.handler.create(invocation) {
            let effect = Arc::new(effect);

            if invocation.managed() {
                let mut managed = self.managed.write();
                managed.push(effect.clone());
            }

            Some(effect)
        } else {
            if invocation.cancelling() {
                self.cancel_effect(invocation);
            }

            None
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
    #[allow(dead_code)]
    fn remove_managed_effect(&self, effect_id: String) {
        let mut managed = self.managed.write();
        if let Some(position) = managed.iter().position(|ef| ef.id() == effect_id) {
            managed.remove(position);
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::event_engine::Event;

    struct TestEvent;

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

        fn run<F>(&self, f: F)
        where
            F: FnOnce(Vec<<Self::Invocation as EffectInvocation>::Event>) + 'static,
        {
            f(vec![]);
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
    fn return_not_managed_effect() {
        let dispatcher = Arc::new(EffectDispatcher::new(TestEffectHandler {}));
        let effect = dispatcher.dispatch(&TestInvocation::One);

        assert_eq!(
            dispatcher.managed.read().len(),
            0,
            "Non managed effects shouldn't be stored"
        );
        assert_eq!(effect.unwrap().id(), "EFFECT_ONE");
    }

    #[tokio::test]
    async fn return_managed_effect() {
        // TODO: now we remove it right away!
        let dispatcher = Arc::new(EffectDispatcher::new(TestEffectHandler {}));
        let effect = dispatcher.dispatch(&TestInvocation::Two);

        assert_eq!(
            dispatcher.managed.read().len(),
            1,
            "Managed effect should be removed on completion"
        );

        assert_eq!(effect.unwrap().id(), "EFFECT_TWO");
    }

    #[test]
    fn cancel_managed_effect() {
        // TODO: now we remove it right away!
        let dispatcher = Arc::new(EffectDispatcher::new(TestEffectHandler {}));
        dispatcher.dispatch(&TestInvocation::Three);
        dispatcher.dispatch(&TestInvocation::CancelThree);

        assert_eq!(
            dispatcher.managed.read().len(),
            0,
            "Managed effect should be cancelled"
        );
    }
}
