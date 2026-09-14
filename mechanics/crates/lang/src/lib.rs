#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)]

pub mod arena;
pub mod ast;
pub mod compiler;
pub mod const_eval;
pub mod index;
pub mod ir;
pub mod layouter;
// pub mod lowerer;
pub mod parser;
pub mod semantic;
pub mod symbols;
pub mod ty;
pub mod vm;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {}

impl Span {
    pub const UNDEFINED: Self = Self {};
}

// pub fn run(source: &'a str) -> Result<'a, ir::Module> {
//     let tu = self.parser.parse(source)?;
//     let index = index::Index::generate(&tu)?;
//     let module = Lowerer::new(&index).lower(tu)?;
//     Ok(module)
// }

fn run_index<'a>(tu: &'a ast::TranslationUnit) -> Result<index::Index<'a>, index::IndexError> {
    index::Index::generate(tu)
}
