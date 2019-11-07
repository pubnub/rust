use std::collections::HashMap;

/// # A `Vec<T>` whose keys increment monotonically
///
/// `MVec<T>` is similar in nature to `Vec<T>`. Elements are indexed by `usize` and it implements a
/// similar API. `MVec` has a trick up its sleeve when an element is removed from the middle of the
/// list; the index for all following elements stays the same!
///
/// Use `MVec<T>` where you would normally want a `Vec<T>` but you need to index elements
/// statically while others can be dynamically removed. IOW, it has a very similar use case to any
/// of the ECS storage types in [specs](https://docs.rs/specs/0.15.1/specs/storage/index.html).
#[derive(Debug)]
pub(crate) struct MVec<T> {
    counter: usize,
    inner: HashMap<usize, T>,
}

#[derive(Debug)]
pub(crate) struct MVecIterMut<'a, T> {
    inner: std::collections::hash_map::ValuesMut<'a, usize, T>,
}

impl<T> MVec<T> {
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    pub(crate) fn counter(&self) -> usize {
        self.counter
    }

    pub(crate) fn push(&mut self, item: T) {
        self.inner.insert(self.counter, item);
        self.counter += 1;
    }

    pub(crate) fn remove(&mut self, index: usize) -> T {
        self.inner
            .remove(&index)
            .unwrap_or_else(|| panic!("Index not found: {}", index))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(crate) fn iter_mut(&mut self) -> MVecIterMut<'_, T> {
        MVecIterMut {
            inner: self.inner.values_mut(),
        }
    }
}

impl<T> Default for MVec<T> {
    fn default() -> Self {
        Self {
            counter: Default::default(),
            inner: HashMap::default(),
        }
    }
}

impl<'a, T> Iterator for MVecIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
