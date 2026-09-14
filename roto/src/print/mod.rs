use crate::{
    ast::ident::Ident,
    label::{LabelRef, LabelStore},
    runtime::Runtime,
    types::{ScopeRef, TypeInfo},
    var::Var,
};

mod lir;
mod mir;
mod types;

pub use self::types::TypeDisplay;

pub struct IrPrinter<'a> {
    pub types: &'a TypeInfo,
    pub labels: &'a LabelStore,
    pub scope: Option<ScopeRef>,

    pub rt: &'a Runtime,
}

pub trait Printable {
    fn print(&self, printer: &IrPrinter) -> String;
}

impl Printable for Ident {
    fn print(&self, _printer: &IrPrinter) -> String {
        self.as_str().into()
    }
}

impl Printable for ScopeRef {
    fn print(&self, printer: &IrPrinter) -> String {
        printer.types.print_scope(*self)
    }
}

impl Printable for LabelRef {
    fn print(&self, printer: &IrPrinter) -> String {
        let mut label = printer.labels.get(*self);
        let mut strings = Vec::new();
        loop {
            let ident = label.ident.print(printer);
            let counter = label.counter;
            strings.push(format!("{ident}_{counter}"));
            let Some(parent) = label.parent else {
                break;
            };
            label = printer.labels.get(parent);
            let Some(_) = label.parent else {
                break;
            };
        }

        strings.reverse();
        strings.join(".")
    }
}

impl Printable for Var {
    fn print(&self, printer: &IrPrinter) -> String {
        let (scope, name) = match *self {
            Self::Ident(scope, name) => (scope, format!("%{}", name.print(printer))),
            Self::Temp(scope, idx) => (scope, format!("%{idx}")),
            Self::Return(scope) => (scope, String::from("%return")),
        };
        if Some(scope) == printer.scope {
            name
        } else {
            format!("{}.{name}", scope.print(printer))
        }
    }
}
