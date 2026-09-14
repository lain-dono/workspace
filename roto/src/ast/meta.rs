use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut, Range},
};

#[derive(Debug, Default)]
pub struct Spans(Vec<Span>);

impl Spans {
    pub fn add<T>(&mut self, span: Span, node: T) -> Meta<T> {
        let id = MetaId(self.0.len());
        self.0.push(span);
        Meta { id, node }
    }

    pub fn get(&self, x: impl Into<MetaId>) -> Span {
        self.0[x.into().0]
    }

    pub fn merge(&mut self, x: impl Into<MetaId>, y: impl Into<MetaId>) -> Span {
        self.get(x).merge(self.get(y))
    }
}

impl<T> From<&Meta<T>> for MetaId {
    fn from(value: &Meta<T>) -> Self {
        value.id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub file: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub fn new(file: usize, value: Range<usize>) -> Self {
        Self {
            file,
            start: value.start,
            end: value.end,
        }
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        assert_eq!(self.file, other.file);
        Self {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MetaId(usize);

impl MetaId {
    pub const INTRINSIC: Self = Self(0);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meta<T> {
    pub id: MetaId,
    pub node: T,
}

impl<T> Meta<T> {
    pub const fn new(id: MetaId, node: T) -> Self {
        Self { id, node }
    }

    pub const fn intrinsic(node: T) -> Self {
        let id = MetaId::INTRINSIC;
        Self { id, node }
    }

    pub fn map<U>(self, map: impl FnOnce(Self) -> U) -> Meta<U> {
        Meta {
            id: self.id,
            node: map(self),
        }
    }

    pub fn map_node<U>(self, map: impl FnOnce(T) -> U) -> Meta<U> {
        Meta {
            id: self.id,
            node: map(self.node),
        }
    }

    pub fn into_inner(self) -> T {
        self.node
    }
}

impl<T: AsRef<U>, U: ?Sized> AsRef<U> for Meta<T> {
    fn as_ref(&self) -> &U {
        self.node.as_ref()
    }
}

impl<T: Borrow<str>> Borrow<str> for Meta<T> {
    fn borrow(&self) -> &str {
        self.node.borrow()
    }
}

impl<T> Deref for Meta<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> DerefMut for Meta<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Meta<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.node.fmt(f)
    }
}
