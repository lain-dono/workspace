use crate::arena::{Arena, Handle, HandleVec};
use crate::ir;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ExpressionKind {
    Const,
    Runtime,
}

#[derive(Debug)]

pub struct ExpressionKindTracker {
    inner: HandleVec<ir::Expr, ExpressionKind>,
}

impl ExpressionKindTracker {
    pub const fn new() -> Self {
        Self {
            inner: HandleVec::new(),
        }
    }

    /// Forces the the expression to not be const
    pub fn force_non_const(&mut self, value: Handle<ir::Expr>) {
        self.inner[value] = ExpressionKind::Runtime;
    }

    pub fn insert(&mut self, value: Handle<ir::Expr>, expr_type: ExpressionKind) {
        self.inner.insert(value, expr_type);
    }

    pub fn is_const(&self, h: Handle<ir::Expr>) -> bool {
        matches!(self.type_of(h), ExpressionKind::Const)
    }

    pub fn is_const_or_override(&self, h: Handle<ir::Expr>) -> bool {
        matches!(self.type_of(h), ExpressionKind::Const)
    }

    fn type_of(&self, value: Handle<ir::Expr>) -> ExpressionKind {
        self.inner[value]
    }

    pub fn from_arena(arena: &Arena<ir::Expr>) -> Self {
        let mut tracker = Self {
            inner: HandleVec::with_capacity(arena.len()),
        };

        for (handle, expr) in arena.iter() {
            tracker
                .inner
                .insert(handle, tracker.type_of_with_expr(expr));
        }

        tracker
    }

    fn type_of_with_expr(&self, expr: &ir::Expr) -> ExpressionKind {
        match *expr {
            /*| Expression::ZeroValue(_)*/
            ir::Expr::Literal(_) | ir::Expr::Constant(_) => ExpressionKind::Const,
            ir::Expr::AccessIndex(base, _) => self.type_of(base),
            ir::Expr::Access(base, index) => self.type_of(base).max(self.type_of(index)),
            ir::Expr::Unary(_, expr) => self.type_of(expr),
            ir::Expr::Binary(_, left, right) => self.type_of(left).max(self.type_of(right)),

            ir::Expr::Select(condition, accept, reject) => self
                .type_of(condition)
                .max(self.type_of(accept))
                .max(self.type_of(reject)),

            // Expression::ArrayLength(expr) => self.type_of(expr),
            _ => ExpressionKind::Runtime,
        }
    }
}
