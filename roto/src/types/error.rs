//! Type errors

use super::{DeclKind, Declaration, ResolvedPath, Type, Typifier, ValueKind};
use crate::{ast, print::TypeDisplay};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Info,
}

/// A label is a bit of text attached to a span
#[derive(Clone, Debug)]
pub struct Label {
    pub level: Level,
    pub id: ast::MetaId,
    pub message: String,
}

impl Label {
    /// Create an error label
    fn error(msg: impl core::fmt::Display, id: ast::MetaId) -> Self {
        Self {
            level: Level::Error,
            id,
            message: msg.to_string(),
        }
    }

    /// Create an info label
    fn info(msg: impl core::fmt::Display, id: ast::MetaId) -> Self {
        Self {
            level: Level::Info,
            id,
            message: msg.to_string(),
        }
    }
}

/// A type error displayed to the user
#[derive(Clone, Debug, thiserror::Error)]
#[error("{description}")]
pub struct TypeError {
    pub description: String,
    pub location: ast::MetaId,
    pub labels: Vec<Label>,
}

impl TypeError {
    /// Catch all error with a basic format with just one label
    pub fn simple(
        description: impl core::fmt::Display,
        msg: impl core::fmt::Display,
        span: ast::MetaId,
    ) -> Self {
        Self {
            description: description.to_string(),
            location: span,
            labels: vec![Label::error(msg, span)],
        }
    }

    pub fn duplicate_fields(field_name: &str, locations: &[ast::MetaId]) -> Self {
        Self {
            description: format!("field `{field_name}` appears multiple times in the same record"),
            location: locations[0],
            labels: locations
                .iter()
                .map(|&span| Label::error(format!("field `{field_name}` declared here"), span))
                .collect(),
        }
    }

    pub fn field_mismatch<'a>(
        location: ast::MetaId,
        invalid: impl IntoIterator<Item = &'a ast::Ident>,
        duplicate: impl IntoIterator<Item = &'a ast::Ident>,
        missing: impl IntoIterator<Item = ast::ident::Ident>,
    ) -> Self {
        let missing: Vec<_> = missing.into_iter().map(ast::ident::Ident::as_str).collect();

        let description = if missing.len() > 1 {
            let fields = join_quoted(missing);
            format!("field mismatch: missing fields {fields} in record literal")
        } else if let [m] = &missing[..] {
            format!("field: mismatch: missing field `{m}` in record literal")
        } else {
            "field mismatch".into()
        };

        let mut labels = vec![Label::error(&description, location)];

        for field in invalid {
            labels.push(Label::info("invalid field", field.id));
        }

        for field in duplicate {
            labels.push(Label::info("duplicate field", field.id));
        }

        Self {
            description,
            location,
            labels,
        }
    }

    pub fn expected_type(ident: &ast::Ident, stub: Declaration) -> Self {
        let kind = describe_declaration(&stub);
        Self {
            description: format!("expected type, but found {kind} `{ident}`"),
            location: ident.id,
            labels: vec![Label::error("expected type", ident.id)],
        }
    }

    pub fn expected_module(ident: &ast::Ident, declaration: Declaration) -> Self {
        let kind = describe_declaration(&declaration);
        Self {
            description: format!("expected a module, but found {kind} `{ident}`"),
            location: ident.id,
            labels: vec![Label::error(
                format!("`{ident}` is a {kind}, not a module"),
                ident.id,
            )],
        }
    }

    pub fn declared_twice(new_declaration: &ast::Ident, old_declaration: ast::MetaId) -> Self {
        let (item_type, name) = if let Some(rest) = new_declaration.as_str().strip_prefix("test#") {
            ("test", rest)
        } else {
            ("item", new_declaration.as_str())
        };

        Self {
            description: format!("{item_type} `{name}` is declared multiple times"),
            location: new_declaration.id,
            labels: vec![
                Label::error(format!("`{name}` redefined here"), new_declaration.id),
                Label::info(
                    format!("`{name}` previously declared here"),
                    old_declaration,
                ),
            ],
        }
    }

    pub fn not_defined(ident: &ast::Ident) -> Self {
        Self {
            description: format!("cannot find value `{ident}` in this scope"),
            location: ident.id,
            labels: vec![Label::error("not found in this scope", ident.id)],
        }
    }

    pub fn number_of_arguments_dont_match(
        call_type: &str,
        method_name: &ast::Ident,
        takes: usize,
        given: usize,
    ) -> Self {
        Self {
            description: format!(
                "{call_type} `{method_name}` takes {takes} arguments but {given} arguments were given"
            ),
            location: method_name.id,
            labels: vec![Label::error(
                format!("takes {takes} arguments but {given} arguments were given"),
                method_name.id,
            )],
        }
    }
}

impl Typifier {
    pub fn error_can_only_match_on_enum(&self, ty: &Type, span: ast::MetaId) -> TypeError {
        TypeError {
            description: format!(
                "cannot match on the type `{}`, \
                because only matching on enums is supported.",
                ty.display(&self.types)
            ),
            location: span,
            labels: vec![Label::error(
                format!("cannot match on type `{}`", ty.display(&self.types)),
                span,
            )],
        }
    }

    pub fn error_variant_does_not_have_fields(&self, variant: &ast::Ident, ty: &Type) -> TypeError {
        TypeError {
            description: format!(
                "pattern has fields, but the variant `{variant}` of `{}` doesn't have one",
                ty.display(&self.types)
            ),
            location: variant.id,
            labels: vec![Label::error("unexpected data field", variant.id)],
        }
    }

    pub fn error_need_arguments_on_pattern(&self, variant: &ast::Ident, ty: &Type) -> TypeError {
        TypeError {
            description: format!(
                "pattern has no arguments, but variant `{variant}` of `{}` does have arguments",
                ty.display(&self.types)
            ),
            location: variant.id,
            labels: vec![Label::error("missing arguments", variant.id)],
        }
    }

    pub fn error_variant_does_not_exist(&self, variant: &ast::Ident, ty: &Type) -> TypeError {
        TypeError {
            description: format!(
                "the variant `{variant}` does not exist on `{}`",
                ty.display(&self.types),
            ),
            location: variant.id,
            labels: vec![Label::error(
                format!("variant does not exist on `{}`", ty.display(&self.types),),
                variant.id,
            )],
        }
    }

    pub fn error_mismatched_types(
        &self,
        expected: &Type,
        got: &Type,
        span: ast::MetaId,
        cause: Option<ast::MetaId>,
    ) -> TypeError {
        let types = &self.types;
        let mut labels = vec![Label::error(
            format!(
                "expected `{}`, found `{}`",
                expected.display(types),
                got.display(types),
            ),
            span,
        )];
        if let Some(span) = cause {
            labels.push(Label::info(
                format!("expected because this is `{}`", expected.display(types)),
                span,
            ));
        }

        TypeError {
            description: "mismatched types".into(),
            location: span,
            labels,
        }
    }

    pub fn error_nonexhaustive_match(
        &self,
        span: ast::MetaId,
        missing_variants: &[ast::ident::Ident],
    ) -> TypeError {
        let missing_variants: Vec<_> = missing_variants.iter().map(AsRef::<str>::as_ref).collect();

        TypeError {
            description: format!(
                "match expression is not exhaustive, missing variants {}",
                join_quoted(&missing_variants)
            ),
            location: span,
            labels: vec![Label::error(
                format!("missing variants {}", join_quoted(missing_variants)),
                span,
            )],
        }
    }

    pub fn error_unreachable_expression<T>(&self, expr: &ast::Meta<T>) -> TypeError {
        TypeError {
            description: "expression is unreachable".into(),
            location: expr.id,
            labels: vec![Label::error("unreachable", expr.id)],
        }
    }

    pub fn error_cannot_diverge_here(
        &self,
        divergence_type: &str,
        expr: &ast::Meta<ast::Expr>,
    ) -> TypeError {
        TypeError {
            description: format!("cannot `{divergence_type}` here"),
            location: expr.id,
            labels: vec![Label::error("not allowed", expr.id)],
        }
    }

    pub fn error_expected_value(&self, ident: &ast::Ident, declaration: &Declaration) -> TypeError {
        let kind = describe_declaration(declaration);
        TypeError {
            description: format!(
                "expected a value, but found {kind} `{}`",
                declaration.name.ident
            ),
            location: ident.id,
            labels: vec![
                Label::error("expected a value", ident.id),
                Label::info(
                    format!("{kind} `{}` defined here", declaration.name.ident),
                    declaration.id,
                ),
            ],
        }
    }

    pub fn error_expected_numeric_value(
        &self,
        expr: &ast::Meta<ast::Expr>,
        ty: &Type,
    ) -> TypeError {
        TypeError {
            description: format!(
                "expected a numeric value, found type `{}`",
                ty.display(&self.types),
            ),
            location: expr.id,
            labels: vec![Label::error("not a numeric value", expr.id)],
        }
    }

    pub fn error_expected_value_path(&self, ident: &ast::Ident, path: &ResolvedPath) -> TypeError {
        let (name, kind) = describe_path(path);
        TypeError {
            description: format!("expected a value, but found {kind} `{name}`"),
            location: ident.id,
            labels: vec![Label::error("expected a value", ident.id)],
        }
    }
    pub fn error_expected_function(
        &self,
        ident: &ast::Ident,
        resolved_path: &ResolvedPath,
    ) -> TypeError {
        let (kind, name) = describe_path(resolved_path);
        TypeError {
            description: format!("expected a function, but found {kind} `{name}`",),
            location: ident.id,
            labels: vec![Label::error("expected a function", ident.id)],
        }
    }

    pub fn error_no_field_on_type(&self, ty: &impl TypeDisplay, ident: &ast::Ident) -> TypeError {
        TypeError::simple(
            format!("no field `{ident}` on type `{}`", ty.display(&self.types)),
            format!("unknown field `{ident}`"),
            ident.id,
        )
    }

    pub fn error_no_method_on_type(&self, ty: &impl TypeDisplay, ident: &ast::Ident) -> TypeError {
        TypeError::simple(
            format!("no method `{ident}` on type `{}`", ty.display(&self.types)),
            format!("unknown method `{ident}`"),
            ident.id,
        )
    }

    pub fn error_no_field_or_method_on_type(
        &self,
        ty: &impl TypeDisplay,
        ident: &ast::Ident,
    ) -> TypeError {
        TypeError::simple(
            format!(
                "no field or method `{ident}` on type `{}`",
                ty.display(&self.types)
            ),
            format!("unknown field or method `{ident}`"),
            ident.id,
        )
    }
}

fn describe_declaration(d: &Declaration) -> &str {
    match &d.kind {
        DeclKind::Value(ValueKind::Local, _) => "variable",
        DeclKind::Value(ValueKind::Constant, _) => "constant",
        DeclKind::Type(..) => "type",
        DeclKind::Function(..) => "function",
        DeclKind::Module => "module",
        DeclKind::Method(..) => "method",
        DeclKind::Variant(..) => "variant",
        DeclKind::TypeParam(..) => "type parameter",
    }
}

fn describe_path(p: &ResolvedPath) -> (&str, ast::ident::Ident) {
    match p {
        ResolvedPath::Function { name, .. } => ("function", name.ident),
        ResolvedPath::Method { name, .. } => ("method", name.ident),
        ResolvedPath::Value(path_value) => match path_value.fields.last() {
            Some(&(ident, _)) => ("field", ident),
            None => ("value", path_value.name.ident),
        },
        ResolvedPath::StaticMethod { name, .. } => ("static method", name.ident),
        ResolvedPath::EnumConstructor { variant, .. } => ("enum constructor", variant.name),
    }
}

fn join_quoted<T: core::fmt::Display>(list: impl IntoIterator<Item = T>) -> String {
    let mut list: Vec<_> = list.into_iter().map(|s| format!("`{s}`")).collect();
    let last_item = list.pop().unwrap();
    if list.is_empty() {
        last_item
    } else {
        format!("{} and {last_item}", list.join(", "))
    }
}
