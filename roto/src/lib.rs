#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// #![allow(clippy::must_use_candidate)]
#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unused_self)]
#![allow(clippy::same_functions_in_if_condition)]

// Needed for the roto macros
extern crate self as roto;

pub mod ast;
#[cfg(feature = "cli")]
mod cli;
mod codegen;
mod file_tree;
mod label;
pub mod lir;
pub mod mir;
mod module;
pub mod pipeline;
mod print;
mod runtime;
pub mod tools;
mod types;

pub mod var;

#[cfg(feature = "cli")]
pub use crate::cli::cli;

pub use self::codegen::{Package, TypedFunc};
pub use self::file_tree::{FileSpec, FileTree, SourceFile};
pub use self::pipeline::{RotoError, RotoReport};
pub use self::runtime::{
    RegistrationError, Runtime,
    basic::{OptionWrapper, Verdict},
    func::RegisterableFn,
    items::{Constant, Function, Impl, Item, Library, Module, Type, Use},
    ty::Reflect,
    val::Val,
};

pub(crate) const FIND_HELP: &str = "\n\
    If you are seeing this error you have found a bug in the Roto compiler.\n\
    Please open an issue at https://github.com/NLnetLabs/roto.";

/// Panic with an internal compiler error
///
/// Calling this macro instead of [`panic!`] signals a bug in the compiler
macro_rules! ice {
    () => {
        panic!("Internal compiler error{}", $crate::FIND_HELP)
    };
    ($s:literal) => {
        panic!("Internal compiler error: {}{}", format!($s), $crate::FIND_HELP)
    };
    ($s:literal, $($t:tt)*) => {
        panic!("Internal compiler error: {}{}", format!($s, $($t)*), $crate::FIND_HELP)
    }
}

pub(crate) use ice;
