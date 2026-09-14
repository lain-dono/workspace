use roto::Runtime;

fn main() -> Result<(), roto::RotoReport> {
    #[cfg(feature = "logger")]
    env_logger::init();

    let runtime = Runtime::builtin();
    let mut pkg = runtime
        .compile("examples/modules")
        .inspect_err(|e| eprintln!("{e}"))?;

    let f = pkg.func::<fn(i32) -> i32>("main").unwrap();

    let x = f.call(4i32);
    println!("main(4) = {x}");
    Ok(())
}
