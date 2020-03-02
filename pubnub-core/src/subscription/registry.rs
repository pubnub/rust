use super::mvec::MVec;
use std::borrow::Borrow;
use std::collections::{hash_map::Entry, HashMap};
use std::hash::Hash;

/// A registry of channels.
#[derive(Debug)]
pub(crate) struct Registry<K, V>
where
    K: Eq + Hash,
{
    pub(super) map: HashMap<K, MVec<V>>,
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

impl<K, V> Registry<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: K, value: V) -> (ID, RegistrationEffect) {
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

    pub fn unregister<Q: ?Sized>(&mut self, name: &Q, id: ID) -> Option<(V, UnregistrationEffect)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
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
    pub fn get_mut<Q: ?Sized>(&mut self, name: &Q) -> Option<&mut MVec<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.get_mut(name)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.keys()
    }
}
