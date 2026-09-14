use crate::arena::FastIndexMap;
use core::{borrow::Borrow, fmt, hash::Hash};

type Scope<Name, Var> = FastIndexMap<Name, Var>;

pub struct ScopeTable<Name, Var> {
    scopes: Vec<Scope<Name, Var>>,
    cursor: usize,
}

impl<Name, Var> Default for ScopeTable<Name, Var> {
    fn default() -> Self {
        Self {
            scopes: vec![FastIndexMap::default()],
            cursor: 1,
        }
    }
}

impl<Name: fmt::Debug, Var: fmt::Debug> fmt::Debug for ScopeTable<Name, Var> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SymbolTable ")?;
        f.debug_list()
            .entries(self.scopes[..self.cursor].iter())
            .finish()
    }
}

impl<Name: Hash + Eq, Var> ScopeTable<Name, Var> {
    pub fn push(&mut self) {
        if self.scopes.len() == self.cursor {
            self.scopes.push(FastIndexMap::default());
        } else {
            assert!(self.scopes[self.cursor].is_empty());
        }
        self.cursor += 1;
    }

    pub fn drain(&mut self) -> std::iter::Rev<indexmap::map::Drain<'_, Name, Var>> {
        self.scopes[self.cursor - 1].drain(..).rev()
    }

    pub fn pop(&mut self) {
        assert!(self.cursor != 1, "Tried to pop the root scope");
        self.cursor -= 1;
    }

    pub fn add(&mut self, name: impl Into<Name>, var: Var) -> Option<Var> {
        self.scopes[self.cursor - 1].insert(name.into(), var)
    }

    pub fn lookup<Q>(&self, name: &Q) -> Option<&Var>
    where
        Name: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut iter = self.scopes[..self.cursor].iter().rev();
        iter.find_map(|scope| scope.get(name))
    }
}
