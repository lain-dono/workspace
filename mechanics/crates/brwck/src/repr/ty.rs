use crate::repr::{Region, StructName};
use std::fmt;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Ref(Region, Box<Self>),
    Mut(Region, Box<Self>),
    Struct(StructName, Vec<TyParam>),
    Bound(usize),
}

impl From<StructName> for Ty {
    fn from(value: StructName) -> Self {
        Self::Struct(value, vec![])
    }
}

impl Ty {
    pub fn ptr_ref(region: impl Into<Region>, ty: impl Into<Self>) -> Self {
        Self::Ref(region.into(), Box::new(ty.into()))
    }

    pub fn ptr_mut(region: impl Into<Region>, ty: impl Into<Self>) -> Self {
        Self::Mut(region.into(), Box::new(ty.into()))
    }

    pub fn struct0(name: impl Into<StructName>) -> Self {
        Self::Struct(name.into(), vec![])
    }

    pub fn struct1(name: impl Into<StructName>, param: impl Into<TyParam>) -> Self {
        Self::Struct(name.into(), vec![param.into()])
    }

    #[must_use]
    pub fn subst(&self, params: &[TyParam]) -> Self {
        match *self {
            Self::Unit => Self::Unit,
            Self::Bound(b) => {
                let index = params.len() - 1 - b;
                match &params[index] {
                    TyParam::Ty(t) => t.clone(),
                    TyParam::Region(r) => {
                        panic!("subst: encountered region {r:?} at index {index} not type")
                    }
                }
            }
            Self::Ref(ref rn, ref t) => Self::Ref(rn.subst(params), Box::new(t.subst(params))),
            Self::Mut(ref rn, ref t) => Self::Mut(rn.subst(params), Box::new(t.subst(params))),
            Self::Struct(s, ref unsubst_params) => {
                Self::Struct(s, unsubst_params.iter().map(|p| p.subst(params)).collect())
            }
        }
    }

    pub fn walk_regions<'a>(&'a self) -> Box<dyn Iterator<Item = Region> + 'a> {
        match *self {
            Self::Unit => Box::new(std::iter::empty()),
            Self::Ref(rn, ref ty) | Self::Mut(rn, ref ty) => {
                Box::new(std::iter::once(rn).chain(ty.walk_regions()))
            }
            Self::Struct(_, ref params) => Box::new(params.iter().flat_map(move |p| match p {
                &TyParam::Region(rn) => Box::new(std::iter::once(rn)),
                TyParam::Ty(ty) => ty.walk_regions(),
            })),
            Self::Bound(_) => panic!("encountered bound type when walking regions"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TyParam {
    Region(Region),
    Ty(Ty),
}

impl<R: Into<Region>> From<R> for TyParam {
    fn from(value: R) -> Self {
        Self::Region(value.into())
    }
}

impl From<Ty> for TyParam {
    fn from(value: Ty) -> Self {
        Self::Ty(value)
    }
}

impl TyParam {
    #[must_use]
    pub fn subst(&self, params: &[Self]) -> Self {
        match self {
            Self::Region(r) => Self::Region(r.subst(params)),
            Self::Ty(t) => Self::Ty(t.subst(params)),
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Ref(region, ty) => write!(f, "&{region} ref {ty}"),
            Self::Mut(region, ty) => write!(f, "&{region} mut {ty}"),

            Self::Struct(StructName(name), params) => {
                write!(f, "{name}<")?;
                for (i, param) in params.iter().enumerate() {
                    match i {
                        0 => write!(f, "{param}")?,
                        _ => write!(f, ", {param}")?,
                    }
                }
                f.write_str(">")
            }
            Self::Bound(i) => write!(f, "@{i}"),
        }
    }
}

impl fmt::Display for TyParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(r) => write!(f, "{r}"),
            Self::Ty(ty) => write!(f, "{ty}"),
        }
    }
}
