use crate::lir;
use cranelift::codegen::ir;
use cranelift::frontend::FuncInstBuilder;
use cranelift::prelude::{FloatCC, InstBuilder as _, IntCC};

impl From<lir::Type> for ir::Type {
    fn from(val: lir::Type) -> Self {
        const { assert!(size_of::<usize>() == 8, "only non-toy arch's are supported") };
        match val {
            lir::Type::I8 | lir::Type::Bool => ir::types::I8,
            lir::Type::I16 => ir::types::I16,
            lir::Type::I32 => ir::types::I32,
            lir::Type::I64 | lir::Type::Ptr => ir::types::I64,
            lir::Type::F32 => ir::types::F32,
            lir::Type::F64 => ir::types::F64,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum ImmOp {
    Band,
    Bor,
    Bxor,

    IEq,
    INe,

    ULt,
    ULe,
    UGt,
    UGe,

    SLt,
    SLe,
    SGt,
    SGe,

    Iadd,
    Imul,
    Irsub,
    Ishl,

    Ushr,
    Sshr,

    Rotl,
    Rotr,

    Sdiv,
    Udiv,
    Srem,
    Urem,
}

impl ImmOp {
    #[must_use]
    pub(super) fn emit(self, ins: FuncInstBuilder<'_, '_>, x: ir::Value, y: i64) -> ir::Value {
        match self {
            Self::Band => ins.band_imm(x, y),
            Self::Bor => ins.bor_imm(x, y),
            Self::Bxor => ins.bxor_imm(x, y),

            Self::IEq => ins.icmp_imm(IntCC::Equal, x, y),
            Self::INe => ins.icmp_imm(IntCC::NotEqual, x, y),

            Self::ULt => ins.icmp_imm(IntCC::UnsignedLessThan, x, y),
            Self::ULe => ins.icmp_imm(IntCC::UnsignedLessThanOrEqual, x, y),
            Self::UGt => ins.icmp_imm(IntCC::UnsignedGreaterThan, x, y),
            Self::UGe => ins.icmp_imm(IntCC::UnsignedGreaterThanOrEqual, x, y),

            Self::SLt => ins.icmp_imm(IntCC::SignedLessThan, x, y),
            Self::SLe => ins.icmp_imm(IntCC::SignedLessThanOrEqual, x, y),
            Self::SGt => ins.icmp_imm(IntCC::SignedGreaterThan, x, y),
            Self::SGe => ins.icmp_imm(IntCC::SignedGreaterThanOrEqual, x, y),

            Self::Iadd => ins.iadd_imm(x, y),
            Self::Imul => ins.imul_imm(x, y),
            Self::Irsub => ins.irsub_imm(x, y),
            Self::Ishl => ins.ishl_imm(x, y),
            Self::Ushr => ins.ushr_imm(x, y),
            Self::Sshr => ins.sshr_imm(x, y),

            Self::Rotl => ins.rotl_imm(x, y),
            Self::Rotr => ins.rotr_imm(x, y),

            Self::Sdiv => ins.sdiv_imm(x, y),
            Self::Udiv => ins.udiv_imm(x, y),
            Self::Srem => ins.srem_imm(x, y),
            Self::Urem => ins.urem_imm(x, y),
        }
    }
}

impl lir::Value {
    pub(super) fn emit(self, ins: FuncInstBuilder<'_, '_>) -> ir::Value {
        let ty = self.ty();
        match self {
            Self::Bool(x) => ins.iconst(ty.into(), x as i64),
            Self::I8(x) => ins.iconst(ty.into(), x as i64),
            Self::I16(x) => ins.iconst(ty.into(), x as i64),
            Self::I32(x) => ins.iconst(ty.into(), x as i64),
            Self::I64(x) => ins.iconst(ty.into(), x),
            Self::F32(x) => ins.f32const(x),
            Self::F64(x) => ins.f64const(x),
            Self::Ptr(x) => ins.iconst(ty.into(), x as i64),
        }
    }
}

impl lir::UnOp {
    #[must_use]
    pub(super) fn emit(self, ins: FuncInstBuilder<'_, '_>, val: ir::Value) -> ir::Value {
        match self {
            Self::Eqz => ImmOp::IEq.emit(ins, val, 0),
            Self::Clz => ins.clz(val),
            Self::Ctz => ins.ctz(val),
            Self::Pop => ins.popcnt(val),
            Self::BNot => ins.bnot(val),
            Self::INeg => ins.ineg(val),
            Self::FNeg => ins.fneg(val),
        }
    }
}

impl lir::BinOp {
    #[must_use]
    pub(super) fn emit(
        self,
        ins: FuncInstBuilder<'_, '_>,
        lhs: ir::Value,
        rhs: ir::Value,
    ) -> ir::Value {
        match self {
            Self::IAdd => ins.iadd(lhs, rhs),
            Self::FAdd => ins.fadd(lhs, rhs),

            Self::ISub => ins.isub(lhs, rhs),
            Self::FSub => ins.fsub(lhs, rhs),

            Self::IMul => ins.imul(lhs, rhs),
            Self::FMul => ins.fmul(lhs, rhs),

            Self::SRem => ins.srem(lhs, rhs),
            Self::URem => ins.urem(lhs, rhs),

            Self::SDiv => ins.sdiv(lhs, rhs),
            Self::UDiv => ins.udiv(lhs, rhs),
            Self::FDiv => ins.fdiv(lhs, rhs),
        }
    }
}

impl lir::Compare {
    #[must_use]
    pub(super) fn emit(
        self,
        ins: FuncInstBuilder<'_, '_>,
        lhs: ir::Value,
        rhs: ir::Value,
    ) -> ir::Value {
        match self {
            Self::IEq => ins.icmp(IntCC::Equal, lhs, rhs),
            Self::INe => ins.icmp(IntCC::NotEqual, lhs, rhs),

            Self::ULt => ins.icmp(IntCC::UnsignedLessThan, lhs, rhs),
            Self::ULe => ins.icmp(IntCC::UnsignedLessThanOrEqual, lhs, rhs),
            Self::UGt => ins.icmp(IntCC::UnsignedGreaterThan, lhs, rhs),
            Self::UGe => ins.icmp(IntCC::UnsignedGreaterThanOrEqual, lhs, rhs),

            Self::SLt => ins.icmp(IntCC::SignedLessThan, lhs, rhs),
            Self::SLe => ins.icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
            Self::SGt => ins.icmp(IntCC::SignedGreaterThan, lhs, rhs),
            Self::SGe => ins.icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),

            Self::FEq => ins.fcmp(FloatCC::Equal, lhs, rhs),
            Self::FNe => ins.fcmp(FloatCC::NotEqual, lhs, rhs),
            Self::FLt => ins.fcmp(FloatCC::LessThan, lhs, rhs),
            Self::FLe => ins.fcmp(FloatCC::LessThanOrEqual, lhs, rhs),
            Self::FGt => ins.fcmp(FloatCC::GreaterThan, lhs, rhs),
            Self::FGe => ins.fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs),
        }
    }
}
