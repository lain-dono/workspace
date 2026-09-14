//! Labels for basic blocks in the IR
//!
//! These labels are designed to be unique, but still have some structure.
//! Technically, a unique number for each label would suffice, but that would
//! make the generated code harder to debug. Therefore, the labels follow
//! a hierarchical scheme, where labels have a name, a counter and optionally
//! a parent. When they are displayed, we join the linked list formed by the
//! parent labels as separated by `::`.

use crate::ast::ident::Ident;
use std::num::NonZeroU32;

/// A label for a basic block
///
/// This should not be constructed directly, but only via a [`LabelStore`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    pub ident: Ident,
    pub parent: Option<LabelRef>,
    pub counter: u32,
}

/// Holds all the information of labels and gives out references
#[derive(Default)]
pub struct LabelStore {
    storage: Vec<Label>,
}

impl LabelStore {
    /// Create a new label with an identifier
    pub fn label(
        &mut self,
        parent: impl Into<Option<LabelRef>>,
        ident: impl Into<Ident>,
    ) -> LabelRef {
        self.append(Label {
            ident: ident.into(),
            parent: parent.into(),
            counter: 0,
        })
    }

    /// Increment the counter of the label by one and return a reference to it
    pub fn next(&mut self, prev: LabelRef) -> LabelRef {
        let prev = &self.storage[prev.get()];
        let counter = prev.counter + 1;
        self.append(Label { counter, ..*prev })
    }

    fn append(&mut self, label: Label) -> LabelRef {
        self.storage.push(label);
        assert!(self.storage.len() < u32::MAX as usize);
        LabelRef(unsafe { NonZeroU32::new_unchecked(self.storage.len() as u32) })
    }

    /// Get the [`Label`] corresponding to a [`LabelRef`]
    pub fn get(&self, label: LabelRef) -> &Label {
        &self.storage[label.get()]
    }
}

/// A reference to a label in the [`LabelStore`]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LabelRef(NonZeroU32);

impl LabelRef {
    /// Return the value of `self` as a [`usize`].
    const fn get(self) -> usize {
        self.0.get() as usize - 1
    }
}
