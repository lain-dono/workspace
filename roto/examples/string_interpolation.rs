use roto::Runtime;

fn main() -> Result<(), roto::RotoReport> {
    let mut runtime = Runtime::builtin();
    runtime.add_io_functions();

    let mut compiled = runtime
        .compile("examples/string_interpolation.roto")
        .inspect_err(|e| eprintln!("{e}"))?;

    let func = compiled
        .func::<fn() -> ()>("main")
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap();

    func.call();

    Ok(())
}
