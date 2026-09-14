use core::{cmp::Ordering, fmt, hash, marker::PhantomData, num::NonZeroU32, ops};
use indexmap::set::IndexSet;

/// An unique index in the arena array that a handle points to.
/// The "non-zero" part ensures that an `Option<Handle<T>>` has
/// the same size and representation as `Handle<T>`.
pub type Index = NonZeroU32;

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq)]
#[error("Handle {index} of {kind} is either not present, or inaccessible yet")]
pub struct BadHandle {
    pub kind: &'static str,
    pub index: usize,
}

/// A strongly typed reference to an arena item.
///
/// A `Handle` value can be used as an index into an [`Arena`] or [`UniqueArena`].
pub struct Handle<T> {
    index: Index,
    marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            marker: self.marker,
        }
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
        self.index.partial_cmp(&other.index)
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
        self.index.hash(hasher)
    }
}

impl<T> Handle<T> {
    #[cfg(test)]
    pub const DUMMY: Self = Self::from_index(unsafe { Index::new_unchecked(u32::MAX) });

    pub const fn from_index(index: Index) -> Self {
        Self {
            index,
            marker: PhantomData,
        }
    }

    /// Returns the zero-based index of this handle.
    pub const fn index(self) -> usize {
        (self.index.get() - 1) as usize
    }

    /// Convert a `usize` index into a `Handle<T>`.
    fn from_usize(index: usize) -> Self {
        let handle_index = u32::try_from(index + 1)
            .ok()
            .and_then(Index::new)
            .expect("Failed to insert into arena. Handle overflows");

        Self::from_index(handle_index)
    }

    /// Convert a `usize` index into a `Handle<T>`, without range checks.
    const unsafe fn from_usize_unchecked(index: usize) -> Self {
        Self::from_index(Index::new_unchecked((index + 1) as u32))
    }
}

/// A strongly typed range of handles.
pub struct Range<T> {
    start: u32,
    end: u32,
    marker: PhantomData<T>,
}

impl<T> Clone for Range<T> {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            marker: self.marker,
        }
    }
}

impl<T> fmt::Debug for Range<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "[{}..{}]", self.start + 1, self.end)
    }
}

impl<T> Iterator for Range<T> {
    type Item = Handle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.start += 1;
            Some(Handle {
                index: Index::new(self.start).unwrap(),
                marker: self.marker,
            })
        } else {
            None
        }
    }
}

/// An arena holding some kind of component (e.g., type, constant,
/// instruction, etc.) that can be referenced.
///
/// Adding new items to the arena produces a strongly-typed [`Handle`].
/// The arena can be indexed using the given handle to obtain
/// a reference to the stored item.
#[derive(Clone, PartialEq)]
pub struct Arena<T> {
    /// Values of this arena.
    data: Vec<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> Arena<T> {
    /// Create a new arena with no initial capacity allocated.
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Extracts the inner vector.
    #[allow(clippy::missing_const_for_fn)] // ignore due to requirement of #![feature(const_precise_live_drops)]
    pub fn into_inner(self) -> Vec<T> {
        self.data
    }

    /// Returns the current number of items stored in this arena.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the arena contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns an iterator over the items stored in this arena, returning both
    /// the item's handle and a reference to it.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Handle<T>, &T)> {
        self.data
            .iter()
            .enumerate()
            .map(|(i, v)| unsafe { (Handle::from_usize_unchecked(i), v) })
    }

    /// Returns a iterator over the items stored in this arena,
    /// returning both the item's handle and a mutable reference to it.
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (Handle<T>, &mut T)> {
        self.data
            .iter_mut()
            .enumerate()
            .map(|(i, v)| unsafe { (Handle::from_usize_unchecked(i), v) })
    }

    /// Adds a new value to the arena, returning a typed handle.
    pub fn append(&mut self, value: T) -> Handle<T> {
        let index = self.data.len();
        self.data.push(value);
        Handle::from_usize(index)
    }

    /// Fetch a handle to an existing type.
    pub fn fetch_if<F: Fn(&T) -> bool>(&self, fun: F) -> Option<Handle<T>> {
        self.data
            .iter()
            .position(fun)
            .map(|index| unsafe { Handle::from_usize_unchecked(index) })
    }

    /// Adds a value with a custom check for uniqueness:
    /// returns a handle pointing to
    /// an existing element if the check succeeds, or adds a new
    /// element otherwise.
    pub fn fetch_if_or_append<F: Fn(&T, &T) -> bool>(&mut self, value: T, fun: F) -> Handle<T> {
        if let Some(index) = self.data.iter().position(|d| fun(d, &value)) {
            unsafe { Handle::from_usize_unchecked(index) }
        } else {
            self.append(value)
        }
    }

    /// Adds a value with a check for uniqueness, where the check is plain comparison.
    pub fn fetch_or_append(&mut self, value: T) -> Handle<T>
    where
        T: PartialEq,
    {
        self.fetch_if_or_append(value, T::eq)
    }

    pub fn try_get(&self, handle: Handle<T>) -> Result<&T, BadHandle> {
        self.data.get(handle.index()).ok_or_else(|| BadHandle {
            kind: std::any::type_name::<T>(),
            index: handle.index(),
        })
    }

    /// Get a mutable reference to an element in the arena.
    pub fn get_mut(&mut self, handle: Handle<T>) -> &mut T {
        self.data.get_mut(handle.index()).unwrap()
    }

    /// Get the range of handles from a particular number of elements to the end.
    pub fn range_from(&self, old_length: usize) -> Range<T> {
        Range {
            start: old_length as u32,
            end: self.data.len() as u32,
            marker: PhantomData,
        }
    }

    /// Clears the arena keeping all allocations
    pub fn clear(&mut self) {
        self.data.clear()
    }
}

impl<T> ops::Index<Handle<T>> for Arena<T> {
    type Output = T;
    fn index(&self, handle: Handle<T>) -> &T {
        &self.data[handle.index()]
    }
}

impl<T> ops::IndexMut<Handle<T>> for Arena<T> {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut T {
        &mut self.data[handle.index()]
    }
}

impl<T> ops::Index<Range<T>> for Arena<T> {
    type Output = [T];
    fn index(&self, range: Range<T>) -> &[T] {
        &self.data[range.start as usize..range.end as usize]
    }
}

/// An arena whose elements are guaranteed to be unique.
///
/// A `UniqueArena` holds a set of unique values of type `T`, each with an
/// associated [`Span`]. Inserting a value returns a `Handle<T>`, which can be
/// used to index the `UniqueArena` and obtain shared access to the `T` element.
/// Access via a `Handle` is an array lookup - no hash lookup is necessary.
///
/// The element type must implement `Eq` and `Hash`. Insertions of equivalent
/// elements, according to `Eq`, all return the same `Handle`.
///
/// Once inserted, elements may not be mutated.
///
/// `UniqueArena` is similar to [`Arena`]: If `Arena` is vector-like,
/// `UniqueArena` is `HashSet`-like.
#[derive(Clone)]
pub struct UniqueArena<T> {
    set: IndexSet<T>,
}

impl<T> UniqueArena<T> {
    /// Create a new arena with no initial capacity allocated.
    pub fn new() -> Self {
        Self {
            set: IndexSet::new(),
        }
    }

    /// Return the current number of items stored in this arena.
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Return `true` if the arena contains no elements.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Clears the arena, keeping all allocations.
    pub fn clear(&mut self) {
        self.set.clear();
    }
}

impl<T: Eq + hash::Hash> UniqueArena<T> {
    /// Returns an iterator over the items stored in this arena, returning both
    /// the item's handle and a reference to it.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Handle<T>, &T)> {
        self.set.iter().enumerate().map(|(i, v)| {
            let position = i + 1;
            let index = unsafe { Index::new_unchecked(position as u32) };
            (Handle::from_index(index), v)
        })
    }

    /// Insert a new value into the arena.
    ///
    /// Return a [`Handle<T>`], which can be used to index this arena to get a
    /// shared reference to the element.
    ///
    /// If this arena already contains an element that is `Eq` to `value`,
    /// return a `Handle` to the existing element, and drop `value`.
    ///
    /// When the `span` feature is enabled, if `value` is inserted into the
    /// arena, associate `span` with it. An element's span can be retrieved with
    /// the [`get_span`] method.
    ///
    /// [`Handle<T>`]: Handle
    /// [`get_span`]: UniqueArena::get_span
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let (index, _added) = self.set.insert_full(value);
        Handle::from_usize(index)
    }

    /// Return this arena's handle for `value`, if present.
    ///
    /// If this arena already contains an element equal to `value`,
    /// return its handle. Otherwise, return `None`.
    pub fn get(&self, value: &T) -> Option<Handle<T>> {
        self.set
            .get_index_of(value)
            .map(|index| unsafe { Handle::from_usize_unchecked(index) })
    }

    /// Return this arena's value at `handle`, if that is a valid handle.
    pub fn get_handle(&self, handle: Handle<T>) -> Result<&T, BadHandle> {
        self.set.get_index(handle.index()).ok_or_else(|| BadHandle {
            kind: std::any::type_name::<T>(),
            index: handle.index(),
        })
    }
}

impl<T> Default for UniqueArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug + Eq + hash::Hash> fmt::Debug for UniqueArena<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> ops::Index<Handle<T>> for UniqueArena<T> {
    type Output = T;
    fn index(&self, handle: Handle<T>) -> &T {
        &self.set[handle.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_non_unique() {
        let mut arena: Arena<u8> = Arena::new();
        let t1 = arena.append(0);
        let t2 = arena.append(0);
        assert!(t1 != t2);
        assert!(arena[t1] == arena[t2]);
    }

    #[test]
    fn append_unique() {
        let mut arena: Arena<u8> = Arena::new();
        let t1 = arena.append(0);
        let t2 = arena.append(1);
        assert!(t1 != t2);
        assert!(arena[t1] != arena[t2]);
    }

    #[test]
    fn fetch_or_append_non_unique() {
        let mut arena: Arena<u8> = Arena::new();
        let t1 = arena.fetch_or_append(0);
        let t2 = arena.fetch_or_append(0);
        assert!(t1 == t2);
        assert!(arena[t1] == arena[t2])
    }

    #[test]
    fn fetch_or_append_unique() {
        let mut arena: Arena<u8> = Arena::new();
        let t1 = arena.fetch_or_append(0);
        let t2 = arena.fetch_or_append(1);
        assert!(t1 != t2);
        assert!(arena[t1] != arena[t2]);
    }
}
