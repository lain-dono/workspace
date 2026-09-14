use crate::arena::{Arena, Handle, HandleVec, Unique};
use crate::{Span, ir};
use core::num::NonZeroU32;

/// When using this type assume no Abstract Int/Float for now
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),

    Int(i64),
    Float(f64),

    I8(i32),
    I16(i32),
    I32(i32),
    I64(i64),

    U8(u32),
    U16(u32),
    U32(u32),
    U64(u64),

    F32(f32),
    F64(f64),
}

impl Literal {
    pub fn scalar(self) -> Scalar {
        match self {
            Self::Bool(_) => Scalar::Bool,
            Self::Int(_) => Scalar::Int,
            Self::Float(_) => Scalar::Float,
            Self::I8(_) => Scalar::I8,
            Self::I16(_) => Scalar::I16,
            Self::I32(_) => Scalar::I32,
            Self::I64(_) => Scalar::I64,
            Self::U8(_) => Scalar::U8,
            Self::U16(_) => Scalar::U16,
            Self::U32(_) => Scalar::U32,
            Self::U64(_) => Scalar::U64,
            Self::F32(_) => Scalar::F32,
            Self::F64(_) => Scalar::F64,
        }
    }

    pub fn ty_inner(self) -> TypeInner {
        TypeInner::Scalar(self.scalar())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scalar {
    Bool,

    Int,
    Float,

    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    F32,
    F64,
}

impl Scalar {
    pub fn width(&self) -> Option<u32> {
        Some(match self {
            Self::Bool => size_of::<bool>() as u32,

            Self::Int | Self::Float => return None,

            Self::I8 => size_of::<i8>() as u32,
            Self::I16 => size_of::<i16>() as u32,
            Self::I32 => size_of::<i32>() as u32,
            Self::I64 => size_of::<i64>() as u32,

            Self::U8 => size_of::<u8>() as u32,
            Self::U16 => size_of::<u16>() as u32,
            Self::U32 => size_of::<u32>() as u32,
            Self::U64 => size_of::<u64>() as u32,

            Self::F32 => size_of::<f32>() as u32,
            Self::F64 => size_of::<f64>() as u32,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Type {
    pub name: Option<String>,
    pub inner: TypeInner,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TypeInner {
    Scalar(Scalar),
    Pointer(Handle<Type>),
    Array(Handle<Type>, Option<NonZeroU32>, u32),
    Struct(Vec<StructMember>, u32),
}

impl TypeInner {
    /// Get the size of this type.
    pub fn size(&self) -> u32 {
        match *self {
            Self::Scalar(scalar) => scalar.width().unwrap(),
            Self::Pointer { .. } => size_of::<*const u8>() as u32,
            Self::Array(_base, Some(count), stride) => count.get() * stride,
            Self::Array(_base, None, stride) => 0,
            Self::Struct(_, span) => span,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StructMember(pub Option<String>, pub Handle<Type>, pub u32);

#[derive(Debug, PartialEq)]
pub enum TypeResolution {
    Handle(Handle<Type>),
    /// only array/pointer/scalar
    Value(TypeInner),
}

impl TypeResolution {
    pub const fn handle(&self) -> Option<Handle<Type>> {
        match *self {
            Self::Handle(handle) => Some(handle),
            Self::Value(_) => None,
        }
    }

    pub fn inner_with<'a>(&'a self, types: &'a Unique<Type>) -> &'a TypeInner {
        match *self {
            Self::Handle(handle) => &types[handle].inner,
            Self::Value(ref inner) => inner,
        }
    }
}

impl Clone for TypeResolution {
    fn clone(&self) -> Self {
        match *self {
            Self::Handle(handle) => Self::Handle(handle),
            Self::Value(ref v) => Self::Value(match *v {
                TypeInner::Scalar(scalar) => TypeInner::Scalar(scalar),
                TypeInner::Pointer(base) => TypeInner::Pointer(base),
                TypeInner::Array(base, size, stride) => TypeInner::Array(base, size, stride),
                TypeInner::Struct(..) => unreachable!("Unexpected clone type: {v:?}"),
            }),
        }
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]

pub enum ResolveError {
    #[error("Index {1} is out of bounds for expression {0:?}")]
    OutOfBoundsIndex(Handle<ir::Expr>, u32),

    #[error("Invalid access into expression {0:?}, non-indexed")]
    InvalidAccess(Handle<ir::Expr>),
    #[error("Invalid access into expression {0:?}, indexed")]
    InvalidAccessIndexed(Handle<ir::Expr>),

    #[error("Invalid sub-access into type {0:?}, non-indexed")]
    InvalidSubAccess(Handle<Type>),
    #[error("Invalid sub-access into type {0:?}, indexed")]
    InvalidSubAccessIndexed(Handle<Type>),

    #[error("Invalid scalar {0:?}")]
    InvalidScalar(Handle<ir::Expr>),
    #[error("Invalid pointer {0:?}")]
    InvalidPointer(Handle<ir::Expr>),

    #[error("Function {name} not defined")]
    FunctionNotDefined { name: String },
    #[error("Function without return type")]
    FunctionReturnsVoid,
    #[error("Incompatible operands: {0}")]
    IncompatibleOperands(String),
    #[error("Function argument {0} doesn't exist")]
    FunctionArgumentNotFound(u32),
}

#[test]
fn test_error_size() {
    assert_eq!(size_of::<ResolveError>(), 32);
}

pub struct ResolveContext<'a> {
    pub constants: &'a Arena<ir::Constant>,
    pub types: &'a Unique<Type>,
    pub local: &'a Arena<ir::LocalVariable>,
    pub funcs: &'a Arena<ir::Func>,
    pub param: &'a [ir::FuncParam],
}

impl<'a> ResolveContext<'a> {
    /// Initialize a resolve context from the module.
    pub const fn with_locals(
        module: &'a ir::Module,
        local: &'a Arena<ir::LocalVariable>,
        param: &'a [ir::FuncParam],
    ) -> Self {
        Self {
            constants: &module.constants,
            types: &module.types,
            funcs: &module.funcs,
            local,
            param,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn resolve(
        &self,
        expr: &ir::Expr,
        past: impl Fn(Handle<ir::Expr>) -> Result<&'a TypeResolution, ResolveError>,
    ) -> Result<TypeResolution, ResolveError> {
        let types = self.types;

        Ok(match *expr {
            ir::Expr::Access(expr, ..) => match *past(expr)?.inner_with(types) {
                TypeInner::Scalar(_) | TypeInner::Struct(_, _) => {
                    return Err(ResolveError::InvalidAccess(expr));
                }
                TypeInner::Pointer(ty) => TypeResolution::Value(match types[ty].inner {
                    TypeInner::Array(base, ..) => TypeInner::Pointer(base),
                    _ => return Err(ResolveError::InvalidSubAccess(ty)),
                }),
                TypeInner::Array(base, _, _) => TypeResolution::Handle(base),
            },
            ir::Expr::AccessIndex(expr, index) => match *past(expr)?.inner_with(types) {
                TypeInner::Scalar(_) => return Err(ResolveError::InvalidAccessIndexed(expr)),
                TypeInner::Array(base, _, _) => TypeResolution::Handle(base),
                TypeInner::Struct(ref members, _) => {
                    let err = ResolveError::OutOfBoundsIndex(expr, index);
                    TypeResolution::Handle(members.get(index as usize).ok_or(err)?.1)
                }
                TypeInner::Pointer(ty) => TypeResolution::Value(match types[ty].inner {
                    TypeInner::Scalar(_) | TypeInner::Pointer(_) => {
                        return Err(ResolveError::InvalidSubAccessIndexed(ty));
                    }
                    TypeInner::Array(base, _, _) => TypeInner::Pointer(base),
                    TypeInner::Struct(ref members, _) => {
                        let err = ResolveError::OutOfBoundsIndex(expr, index);
                        TypeInner::Pointer(members.get(index as usize).ok_or(err)?.1)
                    }
                }),
            },
            ir::Expr::Literal(lit) => TypeResolution::Value(lit.ty_inner()),
            ir::Expr::Constant(h) => TypeResolution::Handle(self.constants[h].ty),
            ir::Expr::FuncParam(index) => {
                let arg = self.param.get(index as usize);
                let arg = arg.ok_or(ResolveError::FunctionArgumentNotFound(index))?;
                TypeResolution::Handle(arg.ty)
            }
            ir::Expr::Variable(h) => TypeResolution::Value(TypeInner::Pointer(self.local[h].ty)),
            ir::Expr::Load(pointer) => match *past(pointer)?.inner_with(types) {
                TypeInner::Pointer(base) => TypeResolution::Handle(base),
                _ => return Err(ResolveError::InvalidPointer(pointer)),
            },
            ir::Expr::Unary(_op, expr) => past(expr)?.clone(),
            ir::Expr::Binary(op, lhs, rhs) => match op {
                ir::BinaryOp::Add | ir::BinaryOp::Sub | ir::BinaryOp::Div | ir::BinaryOp::Rem => {
                    past(lhs)?.clone()
                }
                ir::BinaryOp::Mul => {
                    let (lhs, rhs) = (past(lhs)?, past(rhs)?);
                    match (lhs.inner_with(types), rhs.inner_with(types)) {
                        (&TypeInner::Scalar(_), _) => rhs.clone(),
                        (_, &TypeInner::Scalar(_)) => lhs.clone(),

                        (lhs, rhs) => {
                            return Err(ResolveError::IncompatibleOperands(format!(
                                "{lhs:?} * {rhs:?}"
                            )));
                        }
                    }
                }

                ir::BinaryOp::And | ir::BinaryOp::Eor | ir::BinaryOp::Ior => past(lhs)?.clone(),
                ir::BinaryOp::Shl | ir::BinaryOp::Shr => past(lhs)?.clone(),

                ir::BinaryOp::Eq
                | ir::BinaryOp::Ne
                | ir::BinaryOp::Gt
                | ir::BinaryOp::Ge
                | ir::BinaryOp::Lt
                | ir::BinaryOp::Le => {
                    match [past(lhs)?, past(rhs)?].map(|ty| ty.inner_with(types)) {
                        [TypeInner::Scalar(_), TypeInner::Scalar(_)] => {
                            TypeResolution::Value(TypeInner::Scalar(Scalar::Bool))
                        }

                        [lhs, rhs] => {
                            return Err(ResolveError::IncompatibleOperands(format!(
                                "{op:?}({lhs:?}, {rhs:?})"
                            )));
                        }
                    }
                }
                ir::BinaryOp::LogicAnd | ir::BinaryOp::LogicIor => {
                    match [past(lhs)?, past(rhs)?].map(|ty| ty.inner_with(types)) {
                        [
                            TypeInner::Scalar(Scalar::Bool),
                            TypeInner::Scalar(Scalar::Bool),
                        ] => TypeResolution::Value(TypeInner::Scalar(Scalar::Bool)),
                        [lhs, rhs] => {
                            return Err(ResolveError::IncompatibleOperands(format!(
                                "{op:?}({lhs:?}, {rhs:?})"
                            )));
                        }
                    }
                }
            },

            ir::Expr::Select(_, accept, _) => past(accept)?.clone(),
            ir::Expr::CallResult(function) => self.funcs[function]
                .result
                .map(TypeResolution::Handle)
                .ok_or(ResolveError::FunctionReturnsVoid)?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Typifier {
    mapping: HandleVec<ir::Expr, TypeResolution>,
}

impl Typifier {
    pub const fn new() -> Self {
        Self {
            mapping: HandleVec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.mapping.clear();
    }

    pub fn get<'a>(&'a self, expr: Handle<ir::Expr>, types: &'a Unique<Type>) -> &'a TypeInner {
        self.mapping[expr].inner_with(types)
    }

    pub fn register_type(&self, expr: Handle<ir::Expr>, types: &mut Unique<Type>) -> Handle<Type> {
        match self[expr].clone() {
            TypeResolution::Handle(handle) => handle,
            TypeResolution::Value(inner) => {
                types.insert(Type { name: None, inner }, Span::UNDEFINED)
            }
        }
    }

    pub fn grow(
        &mut self,
        expr: Handle<ir::Expr>,
        arena: &Arena<ir::Expr>,
        ctx: &ResolveContext,
    ) -> Result<(), ResolveError> {
        if self.mapping.len() <= expr.index() {
            for (handle, expr) in arena.iter().skip(self.mapping.len()) {
                // Note: the closure can't `Err` by construction
                // log::debug!("Resolving {:?} = {:?} : {:?}", eh, expr, resolution);
                self.mapping
                    .insert(handle, ctx.resolve(expr, |h| Ok(&self.mapping[h]))?);
            }
        }
        Ok(())
    }

    pub fn invalidate(
        &mut self,
        expr: Handle<ir::Expr>,
        arena: &Arena<ir::Expr>,
        ctx: &ResolveContext,
    ) -> Result<(), ResolveError> {
        if self.mapping.len() <= expr.index() {
            self.grow(expr, arena, ctx)
        } else {
            // Note: the closure can't `Err` by construction
            self.mapping[expr] = ctx.resolve(&arena[expr], |h| Ok(&self.mapping[h]))?;
            Ok(())
        }
    }
}

impl core::ops::Index<Handle<ir::Expr>> for Typifier {
    type Output = TypeResolution;

    fn index(&self, handle: Handle<ir::Expr>) -> &Self::Output {
        &self.mapping[handle]
    }
}
