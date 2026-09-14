use super::{FnBuilder, expr::Emit, expr::EmitResult};
use naga::{
    Expression, Function, Handle, Literal, Module, Scalar, ScalarKind, SpecialTypes,
    SwizzleComponent, Type, TypeInner, VectorSize,
    front::Typifier,
    proc::{ResolveContext, ResolveError},
};

pub struct BaseTypes {
    pub bool: Handle<Type>,
    pub bool2: Handle<Type>,
    pub bool3: Handle<Type>,
    pub bool4: Handle<Type>,

    pub u32: Handle<Type>,
    pub u32x2: Handle<Type>,
    pub u32x3: Handle<Type>,
    pub u32x4: Handle<Type>,

    pub i32: Handle<Type>,
    pub i32x2: Handle<Type>,
    pub i32x3: Handle<Type>,
    pub i32x4: Handle<Type>,

    pub f32: Handle<Type>,
    pub f32x2: Handle<Type>,
    pub f32x3: Handle<Type>,
    pub f32x4: Handle<Type>,
}

impl BaseTypes {
    pub fn new(module: &mut super::ModuleBuilder) -> Self {
        const fn scalar(kind: ScalarKind, width: u8) -> Type {
            let scalar = Scalar { kind, width };
            let inner = TypeInner::Scalar(scalar);
            Type { name: None, inner }
        }

        const fn vector(kind: ScalarKind, size: VectorSize, width: u8) -> Type {
            let scalar = Scalar { kind, width };
            let inner = TypeInner::Vector { size, scalar };
            Type { name: None, inner }
        }

        Self {
            bool: module.insert_type(scalar(ScalarKind::Bool, naga::BOOL_WIDTH)),
            bool2: module.insert_type(vector(ScalarKind::Bool, VectorSize::Bi, naga::BOOL_WIDTH)),
            bool3: module.insert_type(vector(ScalarKind::Bool, VectorSize::Tri, naga::BOOL_WIDTH)),
            bool4: module.insert_type(vector(ScalarKind::Bool, VectorSize::Quad, naga::BOOL_WIDTH)),

            u32: module.insert_type(scalar(ScalarKind::Uint, 4)),
            u32x2: module.insert_type(vector(ScalarKind::Uint, VectorSize::Bi, 4)),
            u32x3: module.insert_type(vector(ScalarKind::Uint, VectorSize::Tri, 4)),
            u32x4: module.insert_type(vector(ScalarKind::Uint, VectorSize::Quad, 4)),

            i32: module.insert_type(scalar(ScalarKind::Sint, 4)),
            i32x2: module.insert_type(vector(ScalarKind::Sint, VectorSize::Bi, 4)),
            i32x3: module.insert_type(vector(ScalarKind::Sint, VectorSize::Tri, 4)),
            i32x4: module.insert_type(vector(ScalarKind::Sint, VectorSize::Quad, 4)),

            f32: module.insert_type(scalar(ScalarKind::Float, 4)),
            f32x2: module.insert_type(vector(ScalarKind::Float, VectorSize::Bi, 4)),
            f32x3: module.insert_type(vector(ScalarKind::Float, VectorSize::Tri, 4)),
            f32x4: module.insert_type(vector(ScalarKind::Float, VectorSize::Quad, 4)),
        }
    }
}

pub fn extract_type<'a>(
    ifier: &'a mut Typifier,
    module: &'a Module,
    function: &'a Function,
    expr_handle: Handle<Expression>,
) -> Result<&'a TypeInner, ResolveError> {
    let ctx = ResolveContext {
        constants: &module.constants,
        types: &module.types,
        global_vars: &module.global_variables,
        local_vars: &function.local_variables,
        functions: &module.functions,
        arguments: &function.arguments,
        special_types: &SpecialTypes::default(),
        overrides: &module.overrides,
    };
    ifier.grow(expr_handle, &function.expressions, &ctx)?;
    Ok(ifier.get(expr_handle, &module.types))
}

/*
pub fn is_scalar(ty: &TypeInner) -> Option<ScalarKind> {
    match ty {
        TypeInner::Scalar { kind, .. } => Some(*kind),
        _ => None,
    }
}

pub fn is_vector(ty: &TypeInner) -> Option<(ScalarKind, VectorSize)> {
    match ty {
        TypeInner::Vector { kind, size, .. } => Some((*kind, *size)),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub enum MatrixKind {
    M2,
    M3,
    M4,
}
*/

#[derive(Clone, Copy)]
pub enum VectorKind {
    V1,
    V2,
    V3,
    V4,
}

impl VectorKind {
    pub fn parse(ty: &TypeInner) -> Option<Self> {
        match ty {
            TypeInner::Scalar { .. } => Some(VectorKind::V1),
            TypeInner::Vector { size, .. } => Some(match size {
                VectorSize::Bi => VectorKind::V2,
                VectorSize::Tri => VectorKind::V3,
                VectorSize::Quad => VectorKind::V4,
            }),
            _ => None,
        }
    }
}

impl VectorKind {
    pub fn min(self, other: Self) -> Self {
        match (self, other) {
            (Self::V1, _) | (_, Self::V1) => Self::V1,
            (Self::V2, _) | (_, Self::V2) => Self::V2,
            (Self::V3, _) | (_, Self::V3) => Self::V3,
            (Self::V4, Self::V4) => Self::V4,
        }
    }

    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (Self::V4, _) | (_, Self::V4) => Self::V4,
            (Self::V3, _) | (_, Self::V3) => Self::V3,
            (Self::V2, _) | (_, Self::V2) => Self::V2,
            (Self::V1, Self::V1) => Self::V1,
        }
    }

    pub fn splat<T: Emit + Copy>(&self, function: &mut FnBuilder, value: T) -> EmitResult {
        match self {
            VectorKind::V1 => value.emit(function),
            VectorKind::V2 => [value, value].emit(function),
            VectorKind::V3 => [value, value, value].emit(function),
            VectorKind::V4 => [value, value, value, value].emit(function),
        }
    }
}

impl<'a, 'storage> FnBuilder<'a, 'storage> {
    pub fn resolve_vector(
        &mut self,
        expr: Handle<Expression>,
        src: VectorKind,
        dst: VectorKind,
    ) -> EmitResult {
        resolve_vector(self, expr, src, dst)
    }

    fn expr(&mut self, expr: impl Emit) -> EmitResult {
        expr.emit(self)
    }
}

pub fn resolve_vector(
    f: &mut FnBuilder,
    base: Handle<Expression>,
    src: VectorKind,
    dst: VectorKind,
) -> EmitResult {
    use VectorKind::*;

    let zero = super::expr::F32(0.0).emit(f)?;
    let one = super::expr::F32(1.0).emit(f)?;

    let xyzw = [
        SwizzleComponent::X,
        SwizzleComponent::Y,
        SwizzleComponent::Z,
        SwizzleComponent::W,
    ];

    Ok(match (src, dst) {
        // same
        (V1, V1) | (V2, V2) | (V3, V3) | (V4, V4) => base,

        // promoting
        (V1, V2) => f.expr([base, zero])?,
        (V1, V3) => f.expr([base, zero, zero])?,
        (V1, V4) => f.expr([base, zero, zero, one])?,

        (V2, V3) => f.expr([
            Expression::AccessIndex { base, index: 0 },
            Expression::AccessIndex { base, index: 1 },
            Expression::Literal(Literal::F32(0.0)),
        ])?,
        (V2, V4) => f.expr([
            Expression::AccessIndex { base, index: 0 },
            Expression::AccessIndex { base, index: 1 },
            Expression::Literal(Literal::F32(0.0)),
            Expression::Literal(Literal::F32(1.0)),
        ])?,
        (V3, V4) => f.expr([
            Expression::AccessIndex { base, index: 0 },
            Expression::AccessIndex { base, index: 1 },
            Expression::AccessIndex { base, index: 2 },
            Expression::Literal(Literal::F32(1.0)),
        ])?,

        // truncating
        (V2, V1) | (V3, V1) | (V4, V1) => f.emit(Expression::AccessIndex { base, index: 0 }),
        (V3, V2) | (V4, V2) => f.emit(Expression::Swizzle {
            vector: base,
            size: naga::VectorSize::Bi,
            pattern: xyzw,
        }),
        (V4, V3) => f.emit(Expression::Swizzle {
            vector: base,
            size: naga::VectorSize::Tri,
            pattern: xyzw,
        }),
    })
}
