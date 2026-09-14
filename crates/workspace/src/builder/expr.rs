use super::FnBuilder;
use naga::{
    BinaryOperator, Expression, Handle, MathFunction, Scalar, ScalarKind, Type, TypeInner,
    UnaryOperator, VectorSize,
};

#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("Port not found")]
    PortNotFound,
    #[error("maybe default")]
    MaybeDefault,
    #[error("fail type")]
    FailType,

    #[error("Resolve {0}")]
    Resolve(#[from] naga::proc::ResolveError),
    #[error("Validation {0}")]
    Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),
    #[error("WGSL {0}")]
    Wgsl(#[from] naga::back::wgsl::Error),
}

pub type EmitResult<T = Handle<Expression>> = Result<T, EmitError>;

pub trait Emit: 'static {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult;

    fn sint(self) -> Cast
    where
        Self: Sized,
    {
        Cast(ScalarKind::Sint, Box::new(self))
    }

    fn uint(self) -> Cast
    where
        Self: Sized,
    {
        Cast(ScalarKind::Uint, Box::new(self))
    }

    fn float(self) -> Cast
    where
        Self: Sized,
    {
        Cast(ScalarKind::Float, Box::new(self))
    }

    fn negate(self) -> Unary
    where
        Self: Sized,
    {
        Unary(UnaryOperator::Negate, Box::new(self))
    }

    fn not(self) -> Unary
    where
        Self: Sized,
    {
        Unary(UnaryOperator::LogicalNot, Box::new(self))
    }
}

macro_rules! impl_emit {
    ($( $name:ident ($self:ident, $function:ident) $body:stmt )+) => {
        $(
            impl Emit for $name {
                fn emit(&$self, $function: &mut FnBuilder) -> EmitResult {
                    $body
                }
            }

            impl_emit!(@ $name Add add Add);
            impl_emit!(@ $name Sub sub Subtract);
            impl_emit!(@ $name Mul mul Multiply);
            impl_emit!(@ $name Div div Divide);
            impl_emit!(@ $name Rem rem Modulo);
            impl_emit!(@ $name BitAnd bitand And);
            impl_emit!(@ $name BitOr bitor InclusiveOr);
            impl_emit!(@ $name BitXor bitxor ExclusiveOr);
            impl_emit!(@ $name Shl shl ShiftLeft);
            impl_emit!(@ $name Shr shr ShiftRight);
        )+
    };

    (@ $for:ident $rust_op:ident $f:ident $naga_op:ident) => {
        impl<RHS: Emit> std::ops::$rust_op<RHS> for $for {
            type Output = Binary;
            fn $f(self, rhs: RHS) -> Self::Output {
                Binary(Box::new(self), BinaryOperator::$naga_op, Box::new(rhs))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct FunctionArgument(pub u32);

#[derive(Clone, Copy)]
pub struct AccessIndex(pub Handle<Expression>, pub u32);

pub struct Math(MathFunction, arrayvec::ArrayVec<Box<dyn Emit>, 4>);
pub struct Cast(ScalarKind, Box<dyn Emit>);
pub struct Binary(Box<dyn Emit>, BinaryOperator, Box<dyn Emit>);
pub struct Unary(UnaryOperator, Box<dyn Emit>);

#[derive(Clone)]
pub struct Expr(pub naga::Expression);

#[derive(Clone, Copy)]
pub struct I32(pub i32);

#[derive(Clone, Copy)]
pub struct U32(pub u32);

#[derive(Clone, Copy)]
pub struct F32(pub f32);

#[derive(Clone, Copy)]
pub struct Bool(pub bool);

#[derive(Clone, Copy)]
pub struct Wrap(pub Handle<Expression>);

pub struct Let {
    name: String,
    expr: Box<dyn Emit>,
}

impl Let {
    pub fn new(name: impl Into<String>, expr: impl Emit) -> Self {
        Self {
            name: name.into(),
            expr: Box::new(expr),
        }
    }
}

impl Emit for Expression {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        Ok(function.emit(self.clone()))
    }
}

impl Emit for Handle<Expression> {
    fn emit(&self, _: &mut FnBuilder) -> EmitResult {
        Ok(*self)
    }
}

impl Emit for naga::Literal {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        Ok(function.expression(Expression::Literal(*self)))
    }
}

impl_emit! {
    Wrap(self, _fn) Ok(self.0)
    Expr(self, function) Ok(function.emit(self.0.clone()))

    I32(self, function) naga::Literal::I32(self.0).emit(function)
    U32(self, function) naga::Literal::U32(self.0).emit(function)
    F32(self, function) naga::Literal::F32(self.0).emit(function)
    Bool(self, function) naga::Literal::Bool(self.0).emit(function)

    AccessIndex(self, function) Ok(function.expression(Expression::AccessIndex { base: self.0, index: self.1 }))
    FunctionArgument(self, function) Ok(function.expression(Expression::FunctionArgument(self.0)))

    Let(self, function) {
        let expr = self.expr.emit(function)?;
        function.insert_expression_name(expr, &self.name);
        Ok(expr)
    }

    Math(self, function) {
        let arg = &self.1;
        let expr = Expression::Math {
            fun: self.0,
            arg: arg[0].emit(function)?,
            arg1: (arg.len() > 1).then(|| arg[1].emit(function)).transpose()?,
            arg2: (arg.len() > 2).then(|| arg[2].emit(function)).transpose()?,
            arg3: (arg.len() > 3).then(|| arg[3].emit(function)).transpose()?,
        };
        Ok(function.emit(expr))
    }
    Cast(self, function) {
        let expr = self.1.emit(function)?;
        Ok(function.emit(Expression::As {
            expr,
            kind: self.0,
            convert: Some(4),
        }))
    }
    Binary(self, function) {
        let left = self.0.emit(function)?;
        let right = self.2.emit(function)?;
        Ok(function.emit(Expression::Binary { left, op: self.1, right }))
    }
    Unary(self, function) {
        let expr = self.1.emit(function)?;
        Ok(function.emit(Expression::Unary { op: self.0, expr }))
    }
}

impl<T: Emit> Emit for [T; 2] {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let components = vec![self[0].emit(function)?, self[1].emit(function)?];
        let ty = function.extract_type(components[0])?;
        let inner = vec_inner(VectorSize::Quad, ty.scalar_kind().unwrap());
        let ty = function.insert_type(Type { name: None, inner });
        Ok(function.emit(Expression::Compose { ty, components }))
    }
}

impl<T: Emit> Emit for [T; 3] {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let components = vec![
            self[0].emit(function)?,
            self[1].emit(function)?,
            self[2].emit(function)?,
        ];
        let ty = function.extract_type(components[0])?;
        let inner = vec_inner(VectorSize::Quad, ty.scalar_kind().unwrap());
        let ty = function.insert_type(Type { name: None, inner });
        Ok(function.emit(Expression::Compose { ty, components }))
    }
}
impl<T: Emit> Emit for [T; 4] {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let components = vec![
            self[0].emit(function)?,
            self[1].emit(function)?,
            self[2].emit(function)?,
            self[3].emit(function)?,
        ];
        let ty = function.extract_type(components[0])?;
        let inner = vec_inner(VectorSize::Quad, ty.scalar_kind().unwrap());
        let ty = function.insert_type(Type { name: None, inner });
        Ok(function.emit(Expression::Compose { ty, components }))
    }
}

fn vec_inner(size: VectorSize, kind: ScalarKind) -> TypeInner {
    let width = match kind {
        ScalarKind::Bool => naga::BOOL_WIDTH,
        _ => 4,
    };
    let scalar = Scalar { kind, width };
    TypeInner::Vector { size, scalar }
}
