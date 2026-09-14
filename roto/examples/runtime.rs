use roto::{Runtime, Val, Verdict};

#[derive(Clone, Copy)]
struct Vec2 {
    x: i32,
    y: i32,
}

fn main() -> Result<(), roto::RotoReport> {
    #[cfg(feature = "logger")]
    env_logger::init();

    let lib = {
        let mut lib = roto::Library::default();
        lib.define_const::<Val<Vec2>>("ZERO", "", Val(Vec2 { x: 0, y: 0 }));
        lib.define_copy_ty::<Val<Vec2>>("Vec2", "Some type I want to register");

        let mut impl_block = roto::Impl::new::<Val<Vec2>>();
        impl_block.define_func("x", "", ["v"], |v: Val<Vec2>| -> i32 { v.x });
        impl_block.define_func("y", "", ["v"], |v: Val<Vec2>| -> i32 { v.y });
        lib.add(impl_block.into());
        lib
    };

    let runtime = Runtime::from_lib(lib).unwrap();

    let mut compiled = runtime
        .compile("examples/runtime.roto")
        .inspect_err(|e| eprintln!("{e}"))?;

    let func = compiled
        .func::<fn(Val<Vec2>) -> Verdict<i32, ()>>("main")
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap();

    for x in 0..20 {
        let vec = Vec2 { x, y: 0 };

        let res = func.call(Val(vec));
        let expected = if x > 10 {
            Verdict::Accept(x * 2)
        } else {
            Verdict::Reject(())
        };
        println!("main({x}) = {res:?}   (expected: {expected:?})");
    }
    Ok(())
}
