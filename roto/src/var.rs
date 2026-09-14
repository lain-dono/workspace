use crate::{ast::ident::Ident, types::ScopeRef};

/// Human-readable place
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Var {
    Ident(ScopeRef, Ident),
    Temp(ScopeRef, usize),
    Return(ScopeRef),
}

impl Var {
    #[must_use]
    pub fn label(self) -> Option<Ident> {
        match self {
            Self::Ident(_, ident) => Some(ident),
            Self::Temp(_, _) | Self::Return(_) => None,
        }
    }
}
