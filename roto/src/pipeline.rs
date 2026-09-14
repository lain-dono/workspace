//! Compiler pipeline that executes multiple compiler stages in sequence

use crate::{
    ast::{ParseError, Span, Spans},
    codegen::{Package, check::FunctionRetrievalError},
    file_tree::SourceFile,
    label::LabelStore,
    lir, mir,
    module::{ModuleTree, Parsed},
    runtime::{Runtime, RuntimeFunctionRef},
    types::{Level, TypeError, TypeInfo},
};
use std::collections::HashMap;

#[cfg(feature = "logger")]
use crate::print::{IrPrinter, Printable};

#[cfg(feature = "logger")]
use log::info;

/// An error from a compilation of a Roto script.
#[derive(Debug)]
pub enum RotoError {
    Read(String, std::io::Error),
    Parse(ParseError),
    Type(TypeError),
    TestsFailed(),
    CouldNotRetrieveFunction(FunctionRetrievalError),
}

/// An error report containing a set of Roto errors.
///
/// The report can be printed with the regular [`core::fmt::Display`].
#[derive(Default, thiserror::Error)]
pub struct RotoReport {
    pub files: Vec<SourceFile>,
    pub(crate) errors: Vec<RotoError>,
    pub spans: Spans,
}

impl core::fmt::Display for RotoReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write(f, true)
    }
}

impl core::fmt::Debug for RotoReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write(f, true)
    }
}

impl RotoReport {
    fn filename(&self, s: Span) -> String {
        self.files[s.file].name.clone()
    }

    pub fn write(&self, mut f: impl core::fmt::Write, color: bool) -> core::fmt::Result {
        use ariadne::{Color, Label, Report, ReportKind};

        let sources = self
            .files
            .iter()
            .map(|s| {
                (
                    s.name.clone(),
                    ariadne::Source::from(s.contents.clone())
                        .with_display_line_offset(s.location_offset),
                )
            })
            .collect();

        let mut file_cache = ariadne::FnCache::new(
            (move |id| Err(Box::new(format!("Failed to fetch source '{id}'"))))
                as fn(&_) -> Result<_, Box<dyn core::fmt::Debug + 'static>>,
        )
        .with_sources(sources);

        let config = ariadne::Config::new().with_color(color);

        for error in &self.errors {
            match error {
                RotoError::Read(name, io) => write!(f, "Could not read file `{name}`: {io}")?,
                RotoError::Parse(error) => {
                    let label_message = error.kind.label();
                    let label = Label::new((
                        self.filename(error.location),
                        error.location.start..error.location.end,
                    ))
                    .with_message(label_message)
                    .with_color(Color::Red);

                    let file = self.filename(error.location);

                    let mut report = Report::build(
                        ReportKind::Error,
                        (file, error.location.start..error.location.end),
                    )
                    .with_config(config)
                    .with_message(format!("Parse error: {error}"))
                    .with_label(label);

                    if let Some(note) = &error.note {
                        report = report.with_note(note);
                    }

                    let report = report.finish();

                    let mut v = Vec::new();
                    report.write(&mut file_cache, &mut v).unwrap();
                    let s = String::from_utf8_lossy(&v);
                    write!(f, "{s}")?;
                }
                RotoError::Type(error) => {
                    let labels = error.labels.iter().map(|l| {
                        let s = self.spans.get(l.id);
                        Label::new((self.filename(s), s.start..s.end))
                            .with_message(&l.message)
                            .with_color(match l.level {
                                Level::Error => Color::Red,
                                Level::Info => Color::Blue,
                            })
                    });

                    let file = self.filename(self.spans.get(error.location));

                    let span = self.spans.get(error.location);
                    let report = Report::build(ReportKind::Error, (file, span.start..span.end))
                        .with_config(config)
                        .with_message(format!("Type error: {}", &error.description))
                        .with_labels(labels)
                        .finish();

                    let mut v = Vec::new();
                    report.write(&mut file_cache, &mut v).unwrap();
                    let s = String::from_utf8_lossy(&v);
                    write!(f, "{s}")?;
                }
                RotoError::TestsFailed() => write!(f, "Tests failed")?,
                RotoError::CouldNotRetrieveFunction(e) => {
                    write!(f, "Could not retrieve function: {e}")?;
                }
            }
        }

        Ok(())
    }
}

impl Parsed {
    pub fn typecheck(self, runtime: &Runtime) -> Result<TypeChecked<'_>, RotoReport> {
        let Parsed {
            file_tree,
            module_tree,
            spans,
        } = self;

        let result = crate::types::typecheck(runtime, &module_tree);

        let types = match result {
            Ok(types) => types,
            Err(error) => {
                return Err(RotoReport {
                    files: file_tree.files,
                    errors: vec![RotoError::Type(error)],
                    spans,
                });
            }
        };

        Ok(TypeChecked {
            module_tree,
            types,
            runtime,
        })
    }
}

/// Compiler stage: loaded, parsed and type checked
pub struct TypeChecked<'r> {
    module_tree: ModuleTree,
    types: TypeInfo,
    runtime: &'r Runtime,
}

impl<'r> TypeChecked<'r> {
    #[must_use]
    pub fn lower_to_mir(&self) -> LoweredToMir<'r> {
        let TypeChecked {
            module_tree,
            types,
            runtime,
        } = self;

        let mut types = types.clone();
        let mut labels = LabelStore::default();
        let ir = mir::Mir::lower(module_tree, runtime, &mut types, &mut labels);

        #[cfg(feature = "logger")]
        {
            if log::log_enabled!(log::Level::Info) {
                let printer = IrPrinter {
                    types: &types,
                    labels: &labels,
                    scope: None,
                    rt: &runtime,
                };
                let s = ir.print(&printer);
                info!("\n{s}");
            }
        }

        LoweredToMir {
            runtime,
            mir: ir,
            labels,
            types,
        }
    }
}

/// Compiler stage: MIR
pub struct LoweredToMir<'r> {
    runtime: &'r Runtime,
    mir: mir::Mir,
    labels: LabelStore,
    types: TypeInfo,
}

impl<'r> LoweredToMir<'r> {
    #[must_use]
    pub fn lower_to_lir(self) -> LoweredToLir<'r> {
        let LoweredToMir {
            runtime,
            mir,
            mut labels,
            mut types,
        } = self;

        let mut rt_functions = HashMap::new();
        let mut ctx = lir::LowerCtx {
            rt: runtime,
            types: &mut types,
            labels: &mut labels,
            rt_functions: &mut rt_functions,
        };
        let ir = lir::Lir::lower(&mut ctx, mir);

        #[cfg(feature = "logger")]
        {
            if log::log_enabled!(log::Level::Info) {
                let printer = IrPrinter {
                    types: &types,
                    labels: &labels,
                    scope: None,
                    rt: &runtime,
                };
                let s = ir.print(&printer);
                info!("\n{s}");
            }
        }

        LoweredToLir {
            runtime,
            ir,
            rt_functions,
            labels,
            types,
        }
    }
}

/// Compiler stage: LIR
pub struct LoweredToLir<'r> {
    runtime: &'r Runtime,
    ir: lir::Lir,
    rt_functions: HashMap<RuntimeFunctionRef, lir::Signature>,
    labels: LabelStore,
    types: TypeInfo,
}

impl LoweredToLir<'_> {
    pub fn eval(&self, mem: &mut lir::Memory, args: Vec<lir::Value>) -> Option<lir::Value> {
        lir::eval(self.runtime, &self.ir.functions, "main", mem, args)
    }

    #[must_use]
    pub fn codegen(self) -> Package {
        Package::codegen(
            self.runtime,
            &self.ir.functions,
            &self.rt_functions,
            self.labels,
            self.types,
        )
    }
}
