use std::{borrow::Cow, collections::BTreeMap, fmt};

mod ty;

pub use self::ty::{Ty, TyParam};

pub type InternedString = Cow<'static, str>;

#[derive(Clone, Debug, Default)]
pub struct Func {
    pub decls: Vec<VariableDecl>,
    pub structs: Vec<StructDecl>,
    pub regions: Vec<RegionDecl>,
    pub data: BTreeMap<BlockName, BlockDecl>,
    pub assertions: Vec<Assertion>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableDecl {
    pub var: Variable,
    pub ty: Ty,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Variable(pub &'static str);

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BorrowKind {
    Ref, // shared
    Mut,
}

impl BorrowKind {
    pub fn variance(self) -> Variance {
        match self {
            Self::Ref => Variance::Co,
            Self::Mut => Variance::In,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Kind {
    Region,
    Type,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Variance {
    Co,
    Contra,
    In,
}

impl Variance {
    #[must_use]
    pub fn invert(self) -> Self {
        match self {
            Self::Co => Self::Contra,
            Self::Contra => Self::Co,
            Self::In => Self::In,
        }
    }

    #[must_use]
    pub fn xform(self, v: Self) -> Self {
        match self {
            Self::Co => v,
            Self::Contra => v.invert(),
            Self::In => Self::In,
        }
    }
}

pub use struct_decl::*;
mod struct_decl {
    use super::{Kind, Ty, Variance};
    use std::fmt;

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct StructDecl {
        pub name: StructName,
        pub params: Vec<StructParam>,
        pub fields: Vec<StructField>,
    }

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct StructName(pub &'static str);

    impl From<&'static str> for StructName {
        fn from(value: &'static str) -> Self {
            Self(value)
        }
    }

    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct StructParam {
        pub kind: Kind,
        pub variance: Variance,
        pub may_dangle: bool,
    }

    impl StructParam {
        pub fn region(variance: Variance, may_dangle: bool) -> Self {
            Self {
                kind: Kind::Region,
                variance,
                may_dangle,
            }
        }

        pub fn ty(variance: Variance, may_dangle: bool) -> Self {
            Self {
                kind: Kind::Type,
                variance,
                may_dangle,
            }
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct StructField {
        pub name: FieldName,
        pub ty: Box<Ty>,
    }

    impl StructField {
        pub fn new(name: &'static str, ty: Ty) -> Self {
            Self {
                name: FieldName(name),
                ty: Box::new(ty),
            }
        }
    }

    #[derive(Clone, Copy, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
    pub struct FieldName(pub &'static str);

    impl FieldName {
        pub const STAR: Self = Self("*");
    }

    impl fmt::Display for FieldName {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(fmt, "{}", self.0)
        }
    }
}

pub use region_decl::*;
mod region_decl {
    use super::TyParam;
    use std::fmt;

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct RegionDecl {
        pub name: RegionName,
        pub outlives: Vec<RegionName>,
    }

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum Region {
        Free(RegionName),
        Bound(usize),
    }

    impl fmt::Display for Region {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Free(r) => write!(f, "{r}"),
                Self::Bound(r) => write!(f, "'{r}"),
            }
        }
    }

    impl From<RegionName> for Region {
        fn from(value: RegionName) -> Self {
            Self::Free(value)
        }
    }

    impl Region {
        #[must_use]
        pub fn subst(&self, params: &[TyParam]) -> Self {
            match self {
                Self::Free(..) => *self,
                &Self::Bound(b) => {
                    let index = params.len() - 1 - b;
                    match &params[index] {
                        &TyParam::Region(r) => r,
                        TyParam::Ty(t) => {
                            panic!("subst: encountered type {t:?} at index {index} not region")
                        }
                    }
                }
            }
        }

        pub fn assert_free(self) -> RegionName {
            match self {
                Self::Free(n) => n,
                Self::Bound(b) => panic!("assert_free: encountered bound region with depth {b}"),
            }
        }
    }

    #[derive(Clone, Copy, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
    pub struct RegionName(pub &'static str);

    impl fmt::Display for RegionName {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(fmt, "'{}", self.0)
        }
    }
}

pub use block::*;
mod block {
    use super::{BorrowKind, FieldName, RegionName, Variable};
    use std::fmt;

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct BlockName(pub &'static str);

    impl fmt::Display for BlockName {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(fmt, "{}", self.0)
        }
    }

    impl BlockName {
        pub const START: Self = Self("START");
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct BlockDecl {
        pub name: BlockName,
        pub actions: Vec<Action>,
        pub successors: Vec<BlockName>,
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct Action {
        pub kind: ActionKind,
        pub should_have_error: Option<ExpectedError>,
    }

    impl fmt::Display for Action {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(err) = self.should_have_error.as_ref() {
                write!(f, "{} {:?}", self.kind, err.string)
            } else {
                write!(f, "{}", self.kind)
            }
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct ExpectedError {
        pub string: String,
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub enum ActionKind {
        Noop,

        Init(Box<Path>, Vec<Box<Path>>), // p = use(...)
        Borrow(Box<Path>, RegionName, BorrowKind, Box<Path>), // p = &'X q
        Assign(Box<Path>, Box<Path>),    // p = q;
        Constraint(Box<Constraint>),     // C
        Use(Box<Path>),                  // use(p);
        Drop(Box<Path>),                 // drop(p);

        /// `StorageDead(v)` indicates that the variable is now out of
        /// scope. This is not counted as a use nor a drop; it basically
        /// just pops the stack space. It *is*, however, important to the
        /// borrow checker.
        StorageDead(Variable),

        /// A synthetic action that is inserted into the basic blocks
        /// representing the end of a skolemized region. There is no
        /// syntax for this sort of "action"; they are created by the NLL
        /// logic in `graph.rs`.
        SkolemizedEnd(RegionName),
    }

    impl fmt::Display for ActionKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Noop => f.write_str(";"),
                Self::Init(path, paths) => write!(f, "init {path} = {paths:?}"),
                Self::Borrow(dst, region, kind, src) => {
                    write!(f, "{dst} = &{region} {kind:?} {src}")
                }
                Self::Assign(dst, src) => write!(f, "{dst} {src}"),
                Self::Constraint(constraint) => write!(f, "{constraint:?}"),
                Self::Use(path) => write!(f, "use({path})"),
                Self::Drop(path) => write!(f, "drop({path})"),
                Self::StorageDead(variable) => write!(f, "storage dead {variable}"),
                Self::SkolemizedEnd(region) => write!(f, "ending {region}"),
            }
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub enum Path {
        // P =
        Var(Variable),                   // v
        Extension(Box<Path>, FieldName), // P.n
    }

    impl From<Variable> for Path {
        fn from(value: Variable) -> Self {
            Self::Var(value)
        }
    }

    impl Path {
        pub fn star_var(var: Variable) -> Self {
            Self::Extension(Box::new(Self::Var(var)), FieldName::STAR)
        }
    }

    impl fmt::Display for Path {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match self {
                Self::Var(v) => write!(f, "{v}"),
                Self::Extension(base, name) => {
                    if name == &FieldName::STAR {
                        write!(f, "*{base}")
                    } else if base.is_deref() {
                        write!(f, "({base}).{name}")
                    } else {
                        write!(f, "{base}.{name}")
                    }
                }
            }
        }
    }

    impl Path {
        pub fn base(&self) -> Variable {
            match self {
                &Self::Var(v) => v,
                Self::Extension(e, _) => e.base(),
            }
        }

        pub fn is_deref(&self) -> bool {
            match self {
                Self::Var(_) => false,
                Self::Extension(_, name) => name == &FieldName::STAR,
            }
        }

        /// If the path is `a.b.c`, returns `a.b.c`, `a.b`, and `a`.
        pub fn prefixes(&self) -> Vec<&Path> {
            let mut this = self;
            let mut result = vec![];
            loop {
                result.push(this);
                match *this {
                    Self::Var(_) => return result,
                    Self::Extension(ref base, _) => this = base,
                }
            }
        }

        /// When you have `p = ...`, which variable is reassigned?
        /// If this is `p = x`, then `x` is. Otherwise, nothing.
        pub fn write_def(&self) -> Option<Variable> {
            match self {
                &Self::Var(v) => Some(v),
                Self::Extension(..) => None,
            }
        }

        /// When you have `p = ...`, which variable is read?
        /// If this is `p = x.0`, then `x` is. Otherwise, nothing.
        pub fn write_use(&self) -> Option<Variable> {
            match self {
                Self::Var(..) => None,
                Self::Extension(..) => Some(self.base()),
            }
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub enum Constraint {
        ForAll(Vec<RegionName>, Box<Constraint>),
        Exists(Vec<RegionName>, Box<Constraint>),
        Implies(Vec<OutlivesConstraint>, Box<Constraint>),
        All(Vec<Constraint>),
        Outlives(OutlivesConstraint),
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct OutlivesConstraint {
        pub sup: RegionName,
        pub sub: RegionName,
    }
}

pub use assertion::*;
mod assertion {
    use super::{BlockName, RegionName, Variable};

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub enum Assertion {
        Eq(RegionName, RegionLiteral),
        In(RegionName, Point),
        NotIn(RegionName, Point),
        Live(Variable, BlockName),
        NotLive(Variable, BlockName),
        RegionLive(RegionName, BlockName),
        RegionNotLive(RegionName, BlockName),
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct Point {
        pub block: PointName,
        pub action: usize,
    }

    impl Point {
        pub fn code(block: BlockName, action: usize) -> Self {
            Self {
                block: PointName::Code(block),
                action,
            }
        }
    }

    #[derive(Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
    pub enum PointName {
        Code(BlockName),
        SkolemizedEnd(RegionName),
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct RegionLiteral {
        pub points: Vec<Point>,
    }
}
