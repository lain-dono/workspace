use crate::types::{MustBeSigned, ResolvedName, Type, TypeDef, TypeInfo, TypeName};
use core::fmt::{Display, Formatter, Result, Write};

pub trait TypeDisplay: Sized {
    fn fmt(&self, types: &TypeInfo, f: &mut Formatter<'_>) -> Result;

    fn display<'a>(&'a self, types: &'a TypeInfo) -> impl Display + 'a {
        from_fn(|f| self.fmt(types, f))
    }
}

impl<T: Display> TypeDisplay for T {
    fn fmt(&self, _types: &TypeInfo, f: &mut Formatter<'_>) -> Result {
        Display::fmt(self, f)
    }
}

// TODO: https://github.com/rust-lang/rust/pull/146099
#[must_use = "returns a type implementing Debug and Display, which do not have any effects unless they are used"]
fn from_fn<F: Fn(&mut Formatter<'_>) -> Result>(f: F) -> FromFn<F> {
    FromFn(f)
}

struct FromFn<F: Fn(&mut Formatter<'_>) -> Result>(F);

impl<F: Fn(&mut Formatter<'_>) -> Result> Display for FromFn<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (self.0)(f)
    }
}

impl TypeDisplay for Type {
    fn fmt(&self, types: &TypeInfo, f: &mut Formatter<'_>) -> Result {
        let fmt_args = |args: &[Type]| {
            let mut iter = args.iter();
            let mut s = String::new();
            if let Some(i) = iter.next() {
                s.write_fmt(format_args!("{}", i.display(types)))?;
            }
            for i in iter {
                s.write_fmt(format_args!(", {}", i.display(types)))?;
            }
            Ok(s)
        };

        match types.resolve_ref(self) {
            Type::Var(_) => write!(f, "_"),
            Type::ExplicitVar(s) => write!(f, "{s}"),
            Type::IVar(_, MustBeSigned::Yes) => write!(f, "{{sint}}"),
            Type::IVar(_, MustBeSigned::No) => write!(f, "{{uint}}"),
            Type::FVar(_) => write!(f, "{{float}}"),
            Type::RecordVar(_, fields) | Type::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(s, t)| format!("{s}: {}", t.display(types)))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{ {fields} }}",)
            }
            Type::Unit => write!(f, "()"),
            Type::Never => write!(f, "!"),
            Type::Function(args, ret) => {
                write!(f, "fn({}) -> {}", fmt_args(args)?, ret.display(types))
            }
            Type::Name(x) => write!(f, "{}", x.display(types)),
        }
    }
}

impl TypeDisplay for TypeDef {
    fn fmt(&self, info: &TypeInfo, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Enum(name, _) | Self::Struct(name, _) => Display::fmt(&name.display(info), f),
            Self::Runtime(name, _) => Display::fmt(&name.display(info), f),
            Self::Scalar(primitive) => Display::fmt(primitive, f),
            Self::Bool => Display::fmt("bool", f),
            Self::String => Display::fmt("String", f),
        }
    }
}

impl TypeDisplay for TypeName {
    fn fmt(&self, types: &TypeInfo, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.name.display(types), f)?;
        let mut args = self.args.iter();
        if let Some(arg) = args.next() {
            f.write_char('[')?;
            Display::fmt(&arg.display(types), f)?;
            for arg in args {
                f.write_char(',')?;
                f.write_char(' ')?;
                Display::fmt(&arg.display(types), f)?;
            }
            f.write_char(']')?;
        }
        Ok(())
    }
}

impl TypeDisplay for ResolvedName {
    fn fmt(&self, types: &TypeInfo, f: &mut Formatter<'_>) -> Result {
        let scope = types.print_scope(self.scope);
        let ident = self.ident;
        if scope.is_empty() {
            write!(f, "{ident}")
        } else {
            write!(f, "{scope}.{ident}")
        }
    }
}
