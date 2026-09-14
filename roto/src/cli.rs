use crate::{FileTree, RotoError, RotoReport, Runtime, tools::print::print_highlighted};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate documentation for the runtime
    Doc {
        #[arg()]
        path: PathBuf,
    },
    /// Type check a script
    Check {
        #[arg()]
        file: PathBuf,
    },
    /// Test a script
    Test {
        #[arg()]
        file: PathBuf,
    },
    /// Run a script's function
    Run {
        #[arg()]
        file: PathBuf,
        #[arg(default_value = "main")]
        function: String,
    },
    /// Print a Roto file with syntax highlighting
    Print {
        #[arg()]
        file: PathBuf,
    },
}

/// Run a basic CLI for a given runtime
///
/// This is useful for providing to users to check their scripts or run their tests
/// with the runtime that the host application provides.
///
/// This CLI provides the following subcommands:
///  - `doc`: generate documentation
///  - `check`: type check a script
///  - `test`: run tests for a script
///  - `run`: run a function of a script
pub fn cli(rt: &Runtime) {
    match cli_inner(rt) {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

fn cli_inner(rt: &Runtime) -> Result<(), RotoReport> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Doc { path } => {
            rt.print_documentation(path).unwrap();
        }
        Command::Check { file } => {
            FileTree::read(file)?.parse()?.typecheck(rt)?;
            println!("All ok!");
        }
        Command::Test { file } => {
            let mut p = FileTree::read(file)?.compile(rt)?;

            if let Err(()) = p.run_tests() {
                return Err(RotoReport {
                    errors: vec![RotoError::TestsFailed()],
                    ..Default::default()
                });
            }
        }
        Command::Run { file, function } => {
            let mut p = FileTree::read(file)?.compile(rt)?;

            let f = p.func::<fn()>(function).map_err(|e| RotoReport {
                errors: vec![RotoError::CouldNotRetrieveFunction(e)],
                ..Default::default()
            })?;

            f.call();
        }
        Command::Print { file } => {
            let s = std::fs::read_to_string(file).unwrap();
            print_highlighted(&s);
        }
    }
    Ok(())
}
