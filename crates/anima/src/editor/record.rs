use std::{collections::VecDeque, num::NonZeroUsize};

use bevy::prelude::Resource;

/// Base functionality for all actions.
pub trait Action: Sized {
    /// The target type.
    type Target;

    /// Applies the action on the target.
    fn apply(&mut self, target: &mut Self::Target);

    /// Restores the state of the target as it was before the action was applied.
    fn undo(&mut self, target: &mut Self::Target);

    /// Reapplies the action on the target.
    ///
    /// The default implementation uses the [`Action::apply`] implementation.
    fn redo(&mut self, target: &mut Self::Target) {
        self.apply(target)
    }
}

/// A record of actions.
///
/// The record can roll the targets state backwards and forwards by using the `undo` and `redo` methods.
#[derive(Clone, Resource)]
pub struct Record<A> {
    entries: VecDeque<A>,
    current: usize,
    limit: NonZeroUsize,
    saved: Option<usize>,
}

impl<A: Action> Default for Record<A> {
    fn default() -> Record<A> {
        Self::new(usize::MAX, true)
    }
}

impl<A: Action> Record<A> {
    pub fn new(limit: usize, saved: bool) -> Self {
        Self {
            entries: VecDeque::new(),
            current: 0,
            limit: NonZeroUsize::new(limit).expect("limit can not be `0`"),
            saved: saved.then_some(0),
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = NonZeroUsize::new(limit).expect("limit can not be `0`");
    }

    /// Returns `true` if the record can undo.
    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    /// Returns `true` if the record can redo.
    pub fn can_redo(&self) -> bool {
        self.current < self.entries.len()
    }

    /// Returns `true` if the target is in a saved state, `false` otherwise.
    pub fn is_saved(&self) -> bool {
        self.saved == Some(self.current)
    }

    /// Pushes the action on top of the record and executes its [`Action::apply`] method.
    pub fn apply(&mut self, target: &mut A::Target, mut action: A) {
        action.apply(target);

        let current = self.current;

        // Pop off all elements after len from record.
        let _tail = self.entries.split_off(current);

        // Check if the saved state was popped off.
        self.saved = self.saved.filter(|&saved| saved <= current);

        // If limit is reached, pop off the first action.
        if self.limit.get() == self.current {
            self.entries.pop_front();
            self.saved = self.saved.and_then(|saved| saved.checked_sub(1));
        } else {
            self.current += 1;
        }
        self.entries.push_back(action);
    }

    /// Calls the [`Action::undo`] method for the active action and sets the previous one as the new active one.
    pub fn undo(&mut self, target: &mut A::Target) {
        if self.can_undo() {
            self.entries[self.current - 1].undo(target);
            self.current -= 1;
        }
    }

    /// Calls the [`Action::redo`] method for the active action and sets the next one as the new active one.
    pub fn redo(&mut self, target: &mut A::Target) {
        if self.can_redo() {
            self.entries[self.current].redo(target);
            self.current += 1;
        }
    }

    /// Marks the target as currently being in a saved or unsaved state.
    pub fn set_saved(&mut self, saved: bool) {
        if saved {
            self.saved = Some(self.current);
        } else {
            self.saved = None;
        }
    }

    /// Removes all actions from the record without undoing them.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.saved = self.is_saved().then_some(0);
        self.current = 0;
    }

    /// Revert the changes done to the target since the saved state.
    pub fn revert(&mut self, target: &mut A::Target) {
        if let Some(saved_current) = self.saved {
            self.goto(target, saved_current);
        }
    }

    /// Repeatedly calls [`Action::undo`] or [`Action::redo`] until the action at `current` is reached.
    pub fn goto(&mut self, target: &mut A::Target, current: usize) {
        if current <= self.entries.len() {
            // Decide if we need to undo or redo to reach current.
            let is_redo = current > self.current;
            let f = if is_redo { Self::redo } else { Self::undo };
            while self.current != current {
                f(self, target);
            }
        }
    }
}
