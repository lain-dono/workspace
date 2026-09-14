use super::{Span, generator::Stmt};
use core::ops::{Deref, DerefMut, RangeBounds};

/// A code block is a vector of statements, with maybe a vector of spans.

#[derive(Debug, Clone, Default)]
pub struct Block {
    body: Vec<Stmt>,
    span_info: Vec<Span>,
}

impl Block {
    pub const fn new() -> Self {
        Self {
            body: Vec::new(),
            span_info: Vec::new(),
        }
    }

    pub fn from_vec(body: Vec<Stmt>) -> Self {
        let span_info = core::iter::repeat_n(Span::default(), body.len()).collect();
        Self { body, span_info }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            body: Vec::with_capacity(capacity),

            span_info: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, end: Stmt, span: Span) {
        self.body.push(end);
        self.span_info.push(span);
    }

    pub fn extend(&mut self, item: Option<(Stmt, Span)>) {
        if let Some((end, span)) = item {
            self.push(end, span)
        }
    }

    pub fn extend_block(&mut self, other: Self) {
        self.span_info.extend(other.span_info);
        self.body.extend(other.body);
    }

    pub fn append(&mut self, other: &mut Self) {
        self.span_info.append(&mut other.span_info);
        self.body.append(&mut other.body);
    }

    pub fn cull<R: RangeBounds<usize> + Clone>(&mut self, range: R) {
        self.span_info.drain(range.clone());
        self.body.drain(range);
    }

    pub fn splice<R: RangeBounds<usize> + Clone>(&mut self, range: R, other: Self) {
        self.span_info.splice(range.clone(), other.span_info);
        self.body.splice(range, other.body);
    }

    pub fn span_into_iter(self) -> impl Iterator<Item = (Stmt, Span)> {
        let Self { body, span_info } = self;
        body.into_iter().zip(span_info)
    }

    pub fn span_iter(&self) -> impl Iterator<Item = (&Stmt, &Span)> {
        let span_iter = self.span_info.iter();
        self.body.iter().zip(span_iter)
    }

    pub fn span_iter_mut(&mut self) -> impl Iterator<Item = (&mut Stmt, Option<&mut Span>)> {
        let span_iter = self.span_info.iter_mut().map(Some);
        self.body.iter_mut().zip(span_iter)
    }

    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }
}

impl Deref for Block {
    type Target = [Stmt];

    fn deref(&self) -> &[Stmt] {
        &self.body
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut [Stmt] {
        &mut self.body
    }
}

impl<'a> IntoIterator for &'a Block {
    type Item = &'a Stmt;

    type IntoIter = core::slice::Iter<'a, Stmt>;

    fn into_iter(self) -> core::slice::Iter<'a, Stmt> {
        self.iter()
    }
}

#[cfg(feature = "deserialize")]
impl<'de> serde::Deserialize<'de> for Block {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_vec(Vec::deserialize(deserializer)?))
    }
}

impl From<Vec<Stmt>> for Block {
    fn from(body: Vec<Stmt>) -> Self {
        Self::from_vec(body)
    }
}
