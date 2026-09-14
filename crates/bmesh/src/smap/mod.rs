// Needed because assigning to non-Copy union is unsafe in stable but not in nightly.
#![allow(unused_unsafe)]

//! Contains the slot map implementation.

use std::collections::TryReserveError;
use std::fmt;
use std::iter::{Enumerate, FusedIterator};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Index, IndexMut};
use std::vec::Vec;

pub use self::key::{Key, KeyData};
use self::util::{Never, UnwrapUnchecked};

// Serialization with serde.
#[cfg(feature = "serde")]
mod serialize;

mod key;

#[cfg(test)]
mod tests;
mod util;

// Storage inside a slot or metadata for the freelist when vacant.
union SlotUnion<T> {
    value: ManuallyDrop<T>,
    next_free: u32,
}

// A slot, which represents storage for a value and a current version.
// Can be occupied or vacant.
struct Slot<T> {
    u: SlotUnion<T>,
    version: u32, // Even = vacant, odd = occupied.
}

// Safe API to read a slot.
enum SlotContent<'a, T: 'a> {
    Occupied(&'a T),
    Vacant(&'a u32),
}

enum SlotContentMut<'a, T: 'a> {
    OccupiedMut(&'a mut T),
    VacantMut(&'a mut u32),
}

use self::SlotContent::{Occupied, Vacant};
use self::SlotContentMut::{OccupiedMut, VacantMut};

impl<T> Slot<T> {
    // Is this slot occupied?
    #[inline(always)]
    pub fn occupied(&self) -> bool {
        !self.version.is_multiple_of(2)
    }

    pub fn get(&self) -> SlotContent<'_, T> {
        unsafe {
            if self.occupied() {
                Occupied(&*self.u.value)
            } else {
                Vacant(&self.u.next_free)
            }
        }
    }

    pub fn get_mut(&mut self) -> SlotContentMut<'_, T> {
        unsafe {
            if self.occupied() {
                OccupiedMut(&mut *self.u.value)
            } else {
                VacantMut(&mut self.u.next_free)
            }
        }
    }
}

impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() && self.occupied() {
            // This is safe because we checked that we're occupied.
            unsafe {
                ManuallyDrop::drop(&mut self.u.value);
            }
        }
    }
}

impl<T: Clone> Clone for Slot<T> {
    fn clone(&self) -> Self {
        Self {
            u: match self.get() {
                Occupied(value) => SlotUnion {
                    value: ManuallyDrop::new(value.clone()),
                },
                Vacant(&next_free) => SlotUnion { next_free },
            },
            version: self.version,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self.get_mut(), source.get()) {
            (OccupiedMut(self_val), Occupied(source_val)) => self_val.clone_from(source_val),
            (VacantMut(self_next_free), Vacant(&source_next_free)) => {
                *self_next_free = source_next_free
            }
            (_, Occupied(value)) => {
                self.u = SlotUnion {
                    value: ManuallyDrop::new(value.clone()),
                }
            }
            (_, Vacant(&next_free)) => self.u = SlotUnion { next_free },
        }
        self.version = source.version;
    }
}

impl<T: fmt::Debug> fmt::Debug for Slot<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = fmt.debug_struct("Slot");
        builder.field("version", &self.version);
        match self.get() {
            Occupied(value) => builder.field("value", value).finish(),
            Vacant(next_free) => builder.field("next_free", next_free).finish(),
        }
    }
}

/// Slot map, storage with stable unique keys.
///
/// See [crate documentation](crate) for more details.
#[derive(Debug)]
pub struct Slots<K: Key, V> {
    slots: Vec<Slot<V>>,
    free_head: u32,
    num_elems: u32,
    key: PhantomData<fn(K) -> K>,
}

impl<K: Key, V> Slots<K, V> {
    /// Constructs a new, empty [`SlotMap`] with a custom key type.
    pub fn with_key() -> Self {
        Self::with_capacity_and_key(0)
    }

    /// Creates an empty [`SlotMap`] with the given capacity and a custom key
    /// type.
    ///
    /// The slot map will not reallocate until it holds at least `capacity`
    /// elements.
    pub fn with_capacity_and_key(capacity: usize) -> Self {
        // Create slots with a sentinel at index 0.
        // We don't actually use the sentinel for anything currently, but
        // HopSlotMap does, and if we want keys to remain valid through
        // conversion we have to have one as well.
        let mut slots = Vec::with_capacity(capacity + 1);
        slots.push(Slot {
            u: SlotUnion { next_free: 0 },
            version: 0,
        });

        Self {
            slots,
            free_head: 1,
            num_elems: 0,
            key: PhantomData,
        }
    }

    /// Returns the number of elements in the slot map.
    pub fn len(&self) -> usize {
        self.num_elems as usize
    }

    /// Returns if the slot map is empty.
    pub fn is_empty(&self) -> bool {
        self.num_elems == 0
    }

    /// Returns the number of elements the [`SlotMap`] can hold without
    /// reallocating.
    pub fn capacity(&self) -> usize {
        // One slot is reserved for the sentinel.
        self.slots.capacity() - 1
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the [`SlotMap`]. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows [`usize`].
    pub fn reserve(&mut self, additional: usize) {
        // One slot is reserved for the sentinel.
        let needed = (self.len() + additional).saturating_sub(self.slots.len() - 1);
        self.slots.reserve(needed);
    }

    /// Tries to reserve capacity for at least `additional` more elements to be
    /// inserted in the [`SlotMap`]. The collection may reserve more space to
    /// avoid frequent reallocations.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        // One slot is reserved for the sentinel.
        let needed = (self.len() + additional).saturating_sub(self.slots.len() - 1);
        self.slots.try_reserve(needed)
    }

    /// Returns [`true`] if the slot map contains `key`.
    pub fn contains_key(&self, key: K) -> bool {
        let kd = key.into();
        self.slots
            .get(kd.index as usize)
            .is_some_and(|slot| slot.version == kd.version.get())
    }

    /// Inserts a value into the slot map. Returns a unique key that can be used
    /// to access this value.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the slot map equals
    /// 2<sup>32</sup> - 2.
    #[inline(always)]
    pub(crate) fn insert(&mut self, value: V) -> K {
        unsafe {
            self.try_insert_with_key::<_, Never>(move |_| Ok(value))
                .unwrap_unchecked_()
        }
    }

    /// Inserts a value given by `f` into the slot map. The key where the
    /// value will be stored is passed into `f`. This is useful to store values
    /// that contain their own key.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the slot map equals
    /// 2<sup>32</sup> - 2.
    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn insert_with_key<F: FnOnce(K) -> V>(&mut self, f: F) -> K {
        unsafe {
            self.try_insert_with_key::<_, Never>(move |k| Ok(f(k)))
                .unwrap_unchecked_()
        }
    }

    /// Inserts a value given by `f` into the slot map. The key where the
    /// value will be stored is passed into `f`. This is useful to store values
    /// that contain their own key.
    ///
    /// If `f` returns `Err`, this method returns the error. The slotmap is untouched.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the slot map equals
    /// 2<sup>32</sup> - 2.
    pub(crate) fn try_insert_with_key<F: FnOnce(K) -> Result<V, E>, E>(
        &mut self,
        f: F,
    ) -> Result<K, E> {
        // In case f panics, we don't make any changes until we have the value.
        let new_num_elems = self.num_elems + 1;
        if new_num_elems == u32::MAX {
            panic!("SlotMap number of elements overflow");
        }

        if let Some(slot) = self.slots.get_mut(self.free_head as usize) {
            let occupied_version = slot.version | 1;
            let kd = KeyData::new(self.free_head, occupied_version);

            // Get value first in case f panics or returns an error.
            let value = f(kd.into())?;

            // Update.
            unsafe {
                self.free_head = slot.u.next_free;
                slot.u.value = ManuallyDrop::new(value);
                slot.version = occupied_version;
            }
            self.num_elems = new_num_elems;
            return Ok(kd.into());
        }

        let version = 1;
        let kd = KeyData::new(self.slots.len() as u32, version);

        // Create new slot before adjusting freelist in case f or the allocation panics or errors.
        self.slots.push(Slot {
            u: SlotUnion {
                value: ManuallyDrop::new(f(kd.into())?),
            },
            version,
        });

        self.free_head = kd.index + 1;
        self.num_elems = new_num_elems;
        Ok(kd.into())
    }

    // Helper function to remove a value from a slot. Safe iff the slot is
    // occupied. Returns the value removed.
    #[inline(always)]
    unsafe fn remove_from_slot(&mut self, index: usize) -> V {
        unsafe {
            // Remove value from slot before overwriting union.
            let slot = self.slots.get_unchecked_mut(index);
            let value = ManuallyDrop::take(&mut slot.u.value);

            // Maintain freelist.
            slot.u.next_free = self.free_head;
            self.free_head = index as u32;
            self.num_elems -= 1;
            slot.version = slot.version.wrapping_add(1);

            value
        }
    }

    /// Removes a key from the slot map, returning the value at the key if the
    /// key was not previously removed.
    pub(crate) fn remove(&mut self, key: K) -> Option<V> {
        let kd = key.into();
        if self.contains_key(key) {
            // This is safe because we know that the slot is occupied.
            Some(unsafe { self.remove_from_slot(kd.index as usize) })
        } else {
            None
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all key-value pairs `(k, v)` such that
    /// `f(k, &mut v)` returns false. This method invalidates any removed keys.
    ///
    /// This function must iterate over all slots, empty or not. In the face of
    /// many deleted elements it can be inefficient.
    #[allow(dead_code)]
    pub(crate) fn retain<F: FnMut(K, &mut V) -> bool>(&mut self, mut f: F) {
        for i in 1..self.slots.len() {
            // This is safe because removing elements does not shrink slots.
            let slot = unsafe { self.slots.get_unchecked_mut(i) };
            let version = slot.version;

            let should_remove = if let OccupiedMut(value) = slot.get_mut() {
                let key = KeyData::new(i as u32, version).into();
                !f(key, value)
            } else {
                false
            };

            if should_remove {
                // This is safe because we know that the slot was occupied.
                unsafe { self.remove_from_slot(i) };
            }
        }
    }

    /// Clears the slot map. Keeps the allocated memory for reuse.
    ///
    /// This function must iterate over all slots, empty or not. In the face of
    /// many deleted elements it can be inefficient.
    pub fn clear(&mut self) {
        self.drain();
    }

    /// Clears the slot map, returning all key-value pairs in arbitrary order as
    /// an iterator. Keeps the allocated memory for reuse.
    ///
    /// When the iterator is dropped all elements in the slot map are removed,
    /// even if the iterator was not fully consumed. If the iterator is not
    /// dropped (using e.g. [`std::mem::forget`]), only the elements that were
    /// iterated over are removed.
    ///
    /// This function must iterate over all slots, empty or not. In the face of
    /// many deleted elements it can be inefficient.
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        Drain { cur: 1, sm: self }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: K) -> Option<&V> {
        let kd = key.into();
        self.slots
            .get(kd.index as usize)
            .filter(|slot| slot.version == kd.version.get())
            .map(|slot| unsafe { &*slot.u.value })
    }

    /// Returns a reference to the value corresponding to the key without
    /// version or bounds checking.
    ///
    /// # Safety
    ///
    /// This should only be used if `contains_key(key)` is true. Otherwise it is
    /// potentially unsafe.
    pub unsafe fn get_unchecked(&self, key: K) -> &V {
        unsafe {
            debug_assert!(self.contains_key(key));
            &self.slots.get_unchecked(key.into().index as usize).u.value
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let kd = key.into();
        self.slots
            .get_mut(kd.index as usize)
            .filter(|slot| slot.version == kd.version.get())
            .map(|slot| unsafe { &mut *slot.u.value })
    }

    /// Returns a mutable reference to the value corresponding to the key
    /// without version or bounds checking.
    ///
    /// # Safety
    ///
    /// This should only be used if `contains_key(key)` is true. Otherwise it is
    /// potentially unsafe.
    pub unsafe fn get_unchecked_mut(&mut self, key: K) -> &mut V {
        unsafe {
            debug_assert!(self.contains_key(key));
            &mut self
                .slots
                .get_unchecked_mut(key.into().index as usize)
                .u
                .value
        }
    }

    /// Returns mutable references to the values corresponding to the given
    /// keys. All keys must be valid and disjoint, otherwise None is returned.
    ///
    /// Requires at least stable Rust version 1.51.
    pub fn get_disjoint_mut<const N: usize>(&mut self, keys: [K; N]) -> Option<[&mut V; N]> {
        // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
        // safe because the type we are claiming to have initialized here is a
        // bunch of `MaybeUninit`s, which do not require initialization.
        let mut ptrs: [MaybeUninit<*mut V>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;
        while i < N {
            let kd = keys[i].into();
            if !self.contains_key(kd.into()) {
                break;
            }

            // This key is valid, and thus the slot is occupied. Temporarily
            // mark it as unoccupied so duplicate keys would show up as invalid.
            // This gives us a linear time disjointness check.
            unsafe {
                let slot = self.slots.get_unchecked_mut(kd.index as usize);
                slot.version ^= 1;
                ptrs[i] = MaybeUninit::new(&mut *slot.u.value);
            }
            i += 1;
        }

        // Undo temporary unoccupied markings.
        for &k in &keys[..i] {
            let idx = k.into().index as usize;
            unsafe {
                self.slots.get_unchecked_mut(idx).version ^= 1;
            }
        }

        if i == N {
            // All were valid and disjoint.
            Some(unsafe { std::mem::transmute_copy::<_, [&mut V; N]>(&ptrs) })
        } else {
            None
        }
    }

    /// Returns mutable references to the values corresponding to the given
    /// keys. All keys must be valid and disjoint.
    ///
    /// Requires at least stable Rust version 1.51.
    ///
    /// # Safety
    ///
    /// This should only be used if `contains_key(key)` is true for every given
    /// key and no two keys are equal. Otherwise it is potentially unsafe.
    pub unsafe fn get_disjoint_unchecked_mut<const N: usize>(
        &mut self,
        keys: [K; N],
    ) -> [&mut V; N] {
        unsafe {
            // Safe, see get_disjoint_mut.
            let mut ptrs: [MaybeUninit<*mut V>; N] = MaybeUninit::uninit().assume_init();
            for i in 0..N {
                ptrs[i] = MaybeUninit::new(self.get_unchecked_mut(keys[i]));
            }
            std::mem::transmute_copy::<_, [&mut V; N]>(&ptrs)
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order. The
    /// iterator element type is `(K, &'a V)`.
    ///
    /// This function must iterate over all slots, empty or not. In the face of
    /// many deleted elements it can be inefficient.
    pub fn iter(&self) -> Iter<'_, K, V> {
        let mut it = self.slots.iter().enumerate();
        it.next(); // Skip sentinel.
        Iter {
            slots: it,
            num_left: self.len(),
            _k: PhantomData,
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order, with
    /// mutable references to the values. The iterator element type is
    /// `(K, &'a mut V)`.
    ///
    /// This function must iterate over all slots, empty or not. In the face of
    /// many deleted elements it can be inefficient.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let len = self.len();
        let mut it = self.slots.iter_mut().enumerate();
        it.next(); // Skip sentinel.
        IterMut {
            num_left: len,
            slots: it,
            _k: PhantomData,
        }
    }
}

impl<K: Key, V> Clone for Slots<K, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            ..*self
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.slots.clone_from(&source.slots);
        self.free_head = source.free_head;
        self.num_elems = source.num_elems;
    }
}

impl<K: Key, V> Default for Slots<K, V> {
    fn default() -> Self {
        Self::with_key()
    }
}

impl<K: Key, V> Index<K> for Slots<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &V {
        self.get(key).expect("invalid SlotMap key used")
    }
}

impl<K: Key, V> IndexMut<K> for Slots<K, V> {
    fn index_mut(&mut self, key: K) -> &mut V {
        self.get_mut(key).expect("invalid SlotMap key used")
    }
}

// Iterators.
/// A draining iterator for [`SlotMap`].
///
/// This iterator is created by [`SlotMap::drain`].
#[derive(Debug)]
pub struct Drain<'a, K: 'a + Key, V: 'a> {
    sm: &'a mut Slots<K, V>,
    cur: usize,
}

/// An iterator that moves key-value pairs out of a [`SlotMap`].
///
/// This iterator is created by calling the `into_iter` method on [`SlotMap`],
/// provided by the [`IntoIterator`] trait.
#[derive(Debug, Clone)]
pub struct IntoIter<K: Key, V> {
    num_left: usize,
    slots: Enumerate<std::vec::IntoIter<Slot<V>>>,
    _k: PhantomData<fn(K) -> K>,
}

/// An iterator over the key-value pairs in a [`SlotMap`].
///
/// This iterator is created by [`SlotMap::iter`].
#[derive(Debug)]
pub struct Iter<'a, K: 'a + Key, V: 'a> {
    num_left: usize,
    slots: Enumerate<std::slice::Iter<'a, Slot<V>>>,
    _k: PhantomData<fn(K) -> K>,
}

impl<'a, K: 'a + Key, V: 'a> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Iter {
            num_left: self.num_left,
            slots: self.slots.clone(),
            _k: self._k,
        }
    }
}

/// A mutable iterator over the key-value pairs in a [`SlotMap`].
///
/// This iterator is created by [`SlotMap::iter_mut`].
#[derive(Debug)]
pub struct IterMut<'a, K: 'a + Key, V: 'a> {
    num_left: usize,
    slots: Enumerate<std::slice::IterMut<'a, Slot<V>>>,
    _k: PhantomData<fn(K) -> K>,
}

impl<K: Key, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        let len = self.sm.slots.len();
        while self.cur < len {
            let idx = self.cur;
            self.cur += 1;

            // This is safe because removing doesn't shrink slots.
            unsafe {
                let slot = self.sm.slots.get_unchecked(idx);
                if slot.occupied() {
                    let kd = KeyData::new(idx as u32, slot.version);
                    return Some((kd.into(), self.sm.remove_from_slot(idx)));
                }
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sm.len(), Some(self.sm.len()))
    }
}

impl<K: Key, V> Drop for Drain<'_, K, V> {
    fn drop(&mut self) {
        self.for_each(|_drop| {});
    }
}

impl<K: Key, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        for (idx, mut slot) in self.slots.by_ref() {
            if slot.occupied() {
                let kd = KeyData::new(idx as u32, slot.version);

                // Prevent dropping after extracting the value.
                slot.version = 0;

                // This is safe because we know the slot was occupied.
                let value = unsafe { ManuallyDrop::take(&mut slot.u.value) };

                self.num_left -= 1;
                return Some((kd.into(), value));
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.num_left, Some(self.num_left))
    }
}

impl<'a, K: Key, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<(K, &'a V)> {
        for (idx, slot) in self.slots.by_ref() {
            if let Occupied(value) = slot.get() {
                let kd = KeyData::new(idx as u32, slot.version);
                self.num_left -= 1;
                return Some((kd.into(), value));
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.num_left, Some(self.num_left))
    }
}

impl<'a, K: Key, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<(K, &'a mut V)> {
        for (idx, slot) in self.slots.by_ref() {
            let version = slot.version;
            if let OccupiedMut(value) = slot.get_mut() {
                let kd = KeyData::new(idx as u32, version);
                self.num_left -= 1;
                return Some((kd.into(), value));
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.num_left, Some(self.num_left))
    }
}

impl<'a, K: Key, V> IntoIterator for &'a Slots<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Key, V> IntoIterator for &'a mut Slots<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: Key, V> IntoIterator for Slots<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        let mut it = self.slots.into_iter().enumerate();
        it.next(); // Skip sentinel.
        IntoIter {
            num_left: len,
            slots: it,
            _k: PhantomData,
        }
    }
}

impl<K: Key, V> FusedIterator for Iter<'_, K, V> {}
impl<K: Key, V> FusedIterator for IterMut<'_, K, V> {}
impl<K: Key, V> FusedIterator for Drain<'_, K, V> {}
impl<K: Key, V> FusedIterator for IntoIter<K, V> {}

impl<K: Key, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K: Key, V> ExactSizeIterator for IterMut<'_, K, V> {}
impl<K: Key, V> ExactSizeIterator for Drain<'_, K, V> {}
impl<K: Key, V> ExactSizeIterator for IntoIter<K, V> {}

impl<K: Key, V> Slots<K, V> {
    pub(crate) fn next_key(&self) -> K {
        // In case f panics, we don't make any changes until we have the value.
        let new_num_elems = self.num_elems + 1;
        if new_num_elems == u32::MAX {
            panic!("SlotMap number of elements overflow");
        }

        match self.slots.get(self.free_head as usize) {
            Some(slot) => KeyData::new(self.free_head, slot.version | 1).into(),
            _ => KeyData::new(self.slots.len() as u32, 1).into(),
        }
    }
}
