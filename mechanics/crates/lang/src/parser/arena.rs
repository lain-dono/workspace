use std::{cmp::Ordering, fmt, hash, marker::PhantomData, num::NonZeroU32, ops};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Index(NonZeroU32);

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl Index {
    /// Construct a [`$name`] whose value is `n`, if possible.
    pub const fn new(n: u32) -> Option<Self> {
        // If `n` is `u32::MAX`, then `n.wrapping_add(1)` is `0`,
        // so `NonZeroU32::new` returns `None` in exactly the case
        // where we must return `None`.

        match NonZeroU32::new(n.wrapping_add(1)) {
            Some(non_zero) => Some(Self(non_zero)),
            None => None,
        }
    }

    pub const unsafe fn new_unchecked(n: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(n + 1) })
    }

    /// Return the value of `self` as a [`u32`].
    pub const fn get(self) -> u32 {
        self.0.get() - 1
    }

    pub fn checked_add(self, n: u32) -> Option<Self> {
        // Adding `n` to `self` produces `u32::MAX` if and only if
        // adding `n` to `self.0` produces `0`. So we can simply
        // call `NonZeroU32::checked_add` and let its check for zero
        // determine whether our add would have produced `u32::MAX`.
        Some(Self(self.0.checked_add(n)?))
    }
}

pub struct Handle<T> {
    index: Index,
    marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "[{}]", self.index)
    }
}

impl<T> hash::Hash for Handle<T> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher);
    }
}

impl<T> Handle<T> {
    const fn new(index: Index) -> Self {
        Self {
            index,
            marker: PhantomData,
        }
    }

    /// Returns the index of this handle.
    pub const fn index(self) -> usize {
        self.index.get() as usize
    }

    /// Convert a `usize` index into a `Handle<T>`.
    pub(crate) fn from_usize(index: usize) -> Self {
        let index = u32::try_from(index).ok().and_then(Index::new);
        Self::new(index.expect("Failed to insert into arena. Handle overflows"))
    }

    /// Convert a `usize` index into a `Handle<T>`, without range checks.
    const unsafe fn from_usize_unchecked(index: usize) -> Self {
        Handle::new(unsafe { Index::new_unchecked(index as u32) })
    }

    /// Write this handle's index to `formatter`, preceded by `prefix`.
    pub fn write_prefixed(&self, f: &mut fmt::Formatter, prefix: &'static str) -> fmt::Result {
        f.write_str(prefix)?;
        <usize as fmt::Display>::fmt(&self.index(), f)
    }
}

#[derive(Clone)]
pub struct UniqueArena<T> {
    memory: indexmap::IndexSet<T, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>,
}

impl<T> Default for UniqueArena<T> {
    fn default() -> Self {
        Self {
            memory: indexmap::IndexSet::default(),
        }
    }
}

impl<T> ops::Index<Handle<T>> for UniqueArena<T> {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &T {
        &self.memory[handle.index()]
    }
}

impl<T: Eq + hash::Hash> UniqueArena<T> {
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let (index, _added) = self.memory.insert_full(value);
        Handle::from_usize(index)
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Arena<T> {
    pub(crate) memory: Vec<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self { memory: Vec::new() }
    }
}

impl<T: fmt::Debug> fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> ops::Index<Handle<T>> for Arena<T> {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &T {
        &self.memory[handle.index()]
    }
}

impl<T> ops::IndexMut<Handle<T>> for Arena<T> {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut T {
        &mut self.memory[handle.index()]
    }
}

impl<T> Arena<T> {
    /// Create a new arena with no initial capacity allocated.
    pub const fn new() -> Self {
        Self { memory: Vec::new() }
    }

    /// Adds a new value to the arena, returning a typed handle.
    pub fn append(&mut self, value: T) -> Handle<T> {
        let index = self.memory.len();
        self.memory.push(value);
        Handle::from_usize(index)
    }

    /// Get a mutable reference to an element in the arena.
    pub fn get_mut(&mut self, handle: Handle<T>) -> &mut T {
        self.memory.get_mut(handle.index()).unwrap()
    }

    /// Returns the current number of items stored in this arena.
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    /// Returns `true` if the arena contains no elements.
    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }

    /// Returns an iterator over the items stored in this arena, returning both
    /// the item's handle and a reference to it.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Handle<T>, &T)> + ExactSizeIterator {
        let iter = self.memory.iter().enumerate();
        iter.map(|(index, value)| unsafe { (Handle::from_usize_unchecked(index), value) })
    }

    /// Returns a iterator over the items stored in this arena,
    /// returning both the item's handle and a mutable reference to it.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (Handle<T>, &mut T)> + ExactSizeIterator {
        let iter = self.memory.iter_mut().enumerate();
        iter.map(|(index, value)| unsafe { (Handle::from_usize_unchecked(index), value) })
    }

    /// Drains the arena, returning an iterator over the items stored.
    pub fn drain(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (Handle<T>, T)> + ExactSizeIterator + use<'_, T> {
        let iter = self.memory.drain(..).enumerate();
        iter.map(|(index, value)| unsafe { (Handle::from_usize_unchecked(index), value) })
    }
}
