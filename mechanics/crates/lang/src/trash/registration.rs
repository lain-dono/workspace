use super::{Context, Operation, ScriptAccess, ScriptError, ScriptResult};
use bevy::{platform::collections::HashMap, ptr::PtrMut, reflect::PartialReflect};
use std::borrow::Cow;

//type Operation = for<'b> super::Operation<Context<'b>>;

#[derive(Default, Clone)]
pub struct Registration {
    storage: Vec<Operation>,
    names: HashMap<Cow<'static, str>, usize>,
}

impl Registration {
    pub fn new() -> Self {
        let mut this = Self::default();

        this.register("nop", nop);
        this.register("dup", op_dup);
        // this.register("push", op_push);
        this.register("access", op_access);
        this.register("branch", op_branch);
        this.register("branch_if", op_branch_if_true);
        this.register("branch_if_not", op_branch_if_false);

        // this.register_unary();
        // this.register_binary();

        this
    }

    pub fn get(&self, index: usize) -> Operation {
        self.storage[index]
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub fn register(&mut self, name: impl Into<Cow<'static, str>>, run: Operation) -> usize {
        let name = name.into();
        assert!(!self.names.contains_key(&name));

        let id = self.storage.len();
        self.storage.push(run);
        self.names.insert(name, id);
        id
    }

    /*
    fn register_unary(&mut self) {
        macro_rules! impl_op {
            (unary  $name:literal $op:ident :: $f:ident [$( $t:ty )+]) => {
                impl_op!(@ $name :: $f(ctx operand _arg) {
                    $( impl_op!(@ operand $t : { ctx.push_alloc(std::ops::$op::$f(*operand)); }); )+
                });
            };
            (@ $name:literal :: $f:ident ($ctx:ident $operand:ident $arg:ident) $body:tt) => {{
                fn $f(mut $ctx: Context, $arg: &dyn PartialReflect) -> ScriptResult {
                    let $operand = $ctx.pop()?;
                    $body
                    Err(ScriptError::TypeMismatch)
                }
                self.register($name, $f);
            }};
            (@ $operand:ident $t:ty : $body:tt) => {
                if $operand.represents::<$t>() {
                    let $operand = $operand.try_downcast_ref::<$t>().unwrap();
                    return Ok($body);
                }
            };
        }

        impl_op!(unary "neg" Neg::neg   [i8 i16 i32 i64 i128 isize                              f32 f64]);
        impl_op!(unary "not" Not::not   [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   bool]);
    }

    fn register_binary(&mut self) {
        macro_rules! impl_op {
            (bin $name:literal $op:ident :: $f:ident [$( $t:ty )+]) => {
                impl_op!(@ $name :: $f(ctx lhs rhs _arg) {
                    $( impl_op!(@ lhs rhs $t : { ctx.push_alloc(std::ops::$op::<$t>::$f(*lhs, *rhs)); }); )+
                });
            };
            (cmp $cmp:literal $branch:literal $f:ident [$( $t:ty )+]) => {
                impl_op!(@ $cmp :: $f(ctx lhs rhs _arg) {
                    $( impl_op!(@ lhs rhs $t : { ctx.push_alloc(std::cmp::PartialEq::<$t>::$f(lhs, rhs)); }); )+
                });
                impl_op!(@ $branch :: $f(ctx lhs rhs addr) {
                    $( impl_op!(@ lhs rhs $t : {
                        let cond = std::cmp::PartialEq::<$t>::$f(lhs, rhs);
                        if cond { ctx.jump(*addr.try_downcast_ref().unwrap()); }
                    }); )+
                });
            };
            (ord $ord:literal $branch:literal $f:ident [$( $t:ty )+]) => {
                impl_op!(@ $ord :: $f(ctx lhs rhs _arg) {
                    $( impl_op!(@ lhs rhs $t : { ctx.push_alloc(std::cmp::PartialOrd::<$t>::$f(lhs, rhs)); }); )+
                });
                impl_op!(@ $branch :: $f(ctx lhs rhs addr) {
                    $( impl_op!(@ lhs rhs $t : {
                        let cond = std::cmp::PartialOrd::<$t>::$f(lhs, rhs);
                        if cond { ctx.jump(*addr.try_downcast_ref().unwrap()); }
                    }); )+
                });
            };
            (@ $name:literal :: $f:ident ($ctx:ident $lhs:ident $rhs:ident $arg:ident) $body:tt) => {{
                fn $f(mut $ctx: Context, $arg: &dyn PartialReflect) -> ScriptResult {
                    let [$rhs, $lhs] = [$ctx.pop()?, $ctx.pop()?];
                    $body
                    Err(ScriptError::TypeMismatch)
                }
                self.register($name, $f);
            }};
            (@ $lhs:ident $rhs:ident $t:ty : $body:tt) => {
                if $lhs.represents::<$t>() && $rhs.represents::<$t>() {
                    let $lhs = $lhs.try_downcast_ref::<$t>().unwrap();
                    let $rhs = $rhs.try_downcast_ref::<$t>().unwrap();
                    return Ok($body);
                }
            };
        }

        impl_op!(bin "add" Add::add         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(bin "sub" Sub::sub         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(bin "div" Div::div         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(bin "mul" Mul::mul         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(bin "rem" Rem::rem         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);

        impl_op!(bin "shl" Shl::shl         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize]);
        impl_op!(bin "shr" Shr::shr         [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize]);
        impl_op!(bin "and" BitAnd::bitand   [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize]);
        impl_op!(bin "ior" BitOr::bitor     [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize]);
        impl_op!(bin "eor" BitXor::bitxor   [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize]);

        impl_op!(cmp "eq" "branch_eq" eq    [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64 bool]);
        impl_op!(cmp "ne" "branch_ne" ne    [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64 bool]);

        impl_op!(ord "lt" "branch_lt" lt    [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(ord "le" "branch_le" le    [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(ord "gt" "branch_gt" gt    [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
        impl_op!(ord "ge" "branch_ge" ge    [i8 i16 i32 i64 i128 isize  u8 u16 u32 u64 u128 usize   f32 f64]);
    }
    */
}
