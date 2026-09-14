use roto::{Runtime, Val};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct NonEmptyString {
    s: Arc<str>,
}

fn main() -> Result<(), roto::RotoReport> {
    #[cfg(feature = "logger")]
    env_logger::init();

    let lib = {
        let func = |s: Arc<str>| -> Option<Val<NonEmptyString>> {
            if s.is_empty() {
                return None;
            }
            Some(Val(NonEmptyString { s }))
        };

        let mut impl_block = roto::Impl::new::<Val<NonEmptyString>>();
        impl_block.define_func("new", "", ["s"], func);

        let mut lib = roto::Library::default();
        lib.define_clone_ty::<Val<NonEmptyString>>("NonEmptyString", "");
        lib.add(impl_block.into());
        lib
    };

    let rt = Runtime::from_lib(lib).unwrap();

    let mut compiled = rt
        .compile("examples/optional.roto")
        .inspect_err(|e| eprintln!("{e}"))?;

    let func = compiled
        .func::<fn(Arc<str>) -> Option<Val<NonEmptyString>>>("main")
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap();

    let res = func.call("".into());
    println!("main(\"\") = {res:?}");

    let res = func.call("foo".into());
    println!("main(\"foo\") = {res:?}");

    Ok(())
}
