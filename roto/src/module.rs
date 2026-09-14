//! Module tree of a Roto script

use crate::{
    FileTree, RotoError, RotoReport,
    ast::{self, Meta, Parser, Span, Spans, ident::Ident},
};
use std::collections::BTreeMap;

pub struct Parsed {
    pub module_tree: ModuleTree,
    pub file_tree: FileTree,
    pub spans: Spans,
}

#[derive(Default)]
pub struct ModuleTree {
    pub modules: Vec<Module>,
}

#[derive(Clone, Copy)]
pub struct ModuleRef(pub usize);

pub struct Module {
    pub ident: Meta<Ident>,
    pub ast: ast::SyntaxTree,
    pub children: BTreeMap<Ident, ModuleRef>,
    pub parent: Option<ModuleRef>,
}

impl FileTree {
    pub fn parse(self) -> Result<Parsed, RotoReport> {
        Parsed::from_files(self)
    }
}

impl Parsed {
    fn from_files(file_tree: FileTree) -> Result<Self, RotoReport> {
        let mut file_to_mod = BTreeMap::new();
        let mut modules = Vec::new();
        let mut spans = Spans::default();
        let mut errors = Vec::new();

        // First add all modules to the tree
        for (i, file) in file_tree.files.iter().enumerate() {
            let ident = spans.add(
                Span {
                    file: i,
                    start: 0,
                    end: 1,
                },
                Ident::from(&file.module_name),
            );

            let ast = match Parser::parse(i, &mut spans, &file.contents) {
                Ok(ast) => ast,
                Err(err) => {
                    errors.push(RotoError::Parse(err));
                    continue;
                }
            };

            file_to_mod.insert(i, modules.len());

            modules.push(Module {
                ident,
                ast,
                children: BTreeMap::new(),
                parent: None,
            });
        }

        if !errors.is_empty() {
            return Err(RotoReport {
                files: file_tree.files,
                errors,
                spans,
            });
        }

        // Then wire up all the relations between the modules
        for (parent, file) in file_tree.files.iter().enumerate() {
            for &child in &file.children {
                let child_module = &mut modules[child];
                child_module.parent = Some(ModuleRef(parent));
                let ident = *child_module.ident;
                modules[parent].children.insert(ident, ModuleRef(child));
            }
        }

        Ok(Self {
            module_tree: ModuleTree { modules },
            file_tree,
            spans,
        })
    }
}
