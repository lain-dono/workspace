//! Well-typed indices into [`Arena`]s and [`UniqueArena`]s.
//!
//! This module defines [`Handle`] and related types.
//!
//! [`Arena`]: super::Arena
//! [`UniqueArena`]: super::UniqueArena

use super::Index;
use core::{cmp::Ordering, fmt, hash, marker::PhantomData};

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq)]
#[error("Handle {index} of {kind} is either not present, or inaccessible yet")]
pub struct BadHandle {
    pub kind: &'static str,
    pub index: usize,
}

impl BadHandle {
    pub fn new<T>(handle: Handle<T>) -> Self {
        Self {
            kind: core::any::type_name::<T>(),
            index: handle.index(),
        }
    }
}

/// A strongly typed reference to an arena item.
///
/// A `Handle` value can be used as an index into an [`Arena`] or [`UniqueArena`].
///
/// [`Arena`]: super::Arena
/// [`UniqueArena`]: super::UniqueArena
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
    pub(crate) const fn new(index: Index) -> Self {
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
    pub fn from_usize(index: usize) -> Self {
        let index = u32::try_from(index).ok().and_then(Index::new);
        Self::new(index.expect("Failed to insert into arena. Handle overflows"))
    }

    /// Convert a `usize` index into a `Handle<T>`, without range checks.
    pub const unsafe fn from_usize_unchecked(index: usize) -> Self {
        Self::new(unsafe { Index::new_unchecked(index as u32) })
    }

    /// Write this handle's index to `formatter`, preceded by `prefix`.
    pub fn write_prefixed(
        &self,
        formatter: &mut fmt::Formatter,
        prefix: &'static str,
    ) -> fmt::Result {
        formatter.write_str(prefix)?;
        <usize as fmt::Display>::fmt(&self.index(), formatter)
    }
}
