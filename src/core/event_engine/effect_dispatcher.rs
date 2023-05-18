use crate::core::event_engine::{Effect, EffectHandler, EffectInvocation, Event};
use std::{marker::PhantomData, rc::Rc, sync::RwLock};

/// State machine effects dispatcher.
pub struct EffectDispatcher<EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF>,
    EH: EffectHandler<EI, EF>,
    EF: Effect,
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

    _invocation: PhantomData<EI>,
}

impl<EH, EF, EI> EffectDispatcher<EH, EF, EI>
where
    EI: EffectInvocation<Effect = EF>,
    EH: EffectHandler<EI, EF>,
    EF: Effect,
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
    pub fn dispatch<F, E>(&self, invocation: &EI, f: F)
    where
        E: Event,
        F: Fn(Vec<E>),
    {
        if let Some(effect) = self.handler.create(invocation) {
            let effect = Rc::new(effect);

            if invocation.managed() {
                if let Ok(mut managed) = self.managed.write() {
                    managed.push(effect.clone());

                    // Drop as early as possible to avoid deadlock with `write`
                    // lock in `remove_managed_effect`.
                    drop(managed);
                }
            }

            // Placeholder for effect invocation.
            effect.run(|| {
                // Try remove effect from list of managed.
                self.remove_managed_effect(&effect);
                println!("Completed");
            });
        } else if invocation.cancelling() {
            self.cancel_effect(invocation);
        }

        // Placeholder for effect events processing (pass to effects handler).
        f(vec![]);
    }

    /// Handle effect cancellation.
    ///
    /// Effects with managed lifecycle can be cancelled by corresponding effect
    /// invocations.
    fn cancel_effect(&self, invocation: &EI) {
        if let Ok(mut managed) = self.managed.write() {
            if let Some(position) = managed.iter().position(|e| invocation.cancelling_effect(e)) {
                managed.remove(position);
            }

            // Drop as early as possible to avoid deadlocks.
            drop(managed);
        }
    }

    /// Remove managed effect.
    fn remove_managed_effect(&self, effect: &EF) {
        if let Ok(mut managed) = self.managed.write() {
            if let Some(position) = managed.iter().position(|ef| ef.id() == effect.id()) {
                managed.remove(position);
            }

            // Drop as early as possible to avoid deadlocks.
            drop(managed);
        }
    }
}
