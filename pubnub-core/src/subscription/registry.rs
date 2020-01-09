use super::mvec::MVec;
use std::collections::{hash_map::Entry, HashMap};

/// A registry of channels.
#[derive(Debug)]
pub(crate) struct Registry<T> {
    pub(super) map: HashMap<String, MVec<T>>,
}

/// Newtype to protect access to the registry ID.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct ID(usize);

#[derive(Debug)]
pub enum RegistrationEffect {
    NewName,
    ExistingName,
}

#[derive(Debug)]
pub enum UnregistrationEffect {
    NameErased,
    NamePreserved,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<S: Into<String>>(&mut self, name: S, value: T) -> (ID, RegistrationEffect) {
        let entry = self.map.entry(name.into());

        let effect = match &entry {
            Entry::Vacant(_) => RegistrationEffect::NewName,
            Entry::Occupied(_) => RegistrationEffect::ExistingName,
        };

        let mvec = entry.or_default();
        let id = mvec.counter();
        mvec.push(value);

        (ID(id), effect)
    }

    pub fn unregister(&mut self, name: &str, id: ID) -> Option<(T, UnregistrationEffect)> {
        let ID(id) = id;

        let mvec = self.map.get_mut(name)?;
        let removed = mvec.remove(id)?;

        let effect = if mvec.is_empty() {
            self.map.remove(name);
            UnregistrationEffect::NameErased
        } else {
            UnregistrationEffect::NamePreserved
        };

        Some((removed, effect))
    }

    // TODO: provide better interface for iteration over mutable items.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut MVec<T>> {
        self.map.get_mut(name)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
