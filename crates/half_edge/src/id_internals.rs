use sid::{FromUsize, Id, NullId};
use std::marker::PhantomData;

pub type Index = u16;

// We use a magic value to
pub fn is_valid<T>(id: Id<T, Index>) -> bool {
    id.handle != u16::MAX
}

pub struct MagicValueMax<T> {
    marker: PhantomData<T>,
}

impl<T> NullId<Id<T, Index>> for MagicValueMax<T> {
    fn null_id() -> Id<T, Index> {
        FromUsize::from_usize(u16::MAX as usize)
    }
}
