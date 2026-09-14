use core::f32;
use roto::{FileSpec, FileTree, Library, Package, Runtime, Val, Verdict};
use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicUsize, Ordering},
};

macro_rules! src {
    ($code:literal) => {
        roto::FileTree::test_file(file!(), $code, line!() as usize - 1)
    };
}

macro_rules! source_file {
    ($module_name:literal, $code:literal) => {
        roto::SourceFile {
            name: file!().into(),
            module_name: $module_name.into(),
            contents: $code.into(),
            location_offset: line!() as usize - 1,
            children: Vec::new(),
        }
    };
}

#[track_caller]
fn compile(f: FileTree) -> Package {
    let runtime = Runtime::builtin();
    compile_with_runtime(f, runtime)
}

#[track_caller]
fn compile_with_runtime(f: FileTree, runtime: Runtime) -> Package {
    #[cfg(feature = "logger")]
    let _ = env_logger::try_init();

    match f.compile(&runtime) {
        Ok(x) => x,
        Err(err) => {
            println!("{err}");
            panic!("Compilation error: see above");
        }
    }
}

#[test]
fn unit() {
    let mut p = compile(src! {"
        fn unit_expression() {
            if true { () } else { () }
        }
    "});
    p.func::<fn() -> ()>("unit_expression").unwrap().call();

    let mut p = compile(src! {"
        fn unit_type() -> () {
            let x: () = ();
            x
        }
    "});

    p.func::<fn() -> ()>("unit_type").unwrap().call();
}

#[test]
fn verdict() {
    let mut p = compile(src!("filtermap main() { accept }"));
    let f = p.func::<fn() -> Verdict<(), ()>>("main").unwrap();
    assert_eq!(f.call(), Verdict::Accept(()));

    let mut p = compile(src!("filtermap main() { accept; }"));
    let f = p.func::<fn() -> Verdict<(), ()>>("main").unwrap();
    assert_eq!(f.call(), Verdict::Accept(()));

    let mut p = compile(src!("filtermap main() { reject }"));
    let f = p.func::<fn() -> Verdict<(), ()>>("main").unwrap();
    assert_eq!(f.call(), Verdict::Reject(()));

    let mut p = compile(src!("filtermap main() { reject; }"));
    let f = p.func::<fn() -> Verdict<(), ()>>("main").unwrap();
    assert_eq!(f.call(), Verdict::Reject(()));
}

#[test]
fn equal_to_10() {
    let mut p = compile(src! {"
        filtermap main(x: u32) {
            if x == 10 {
                accept
            } else {
                reject
            }
        }
    "});
    let f = p.func::<fn(u32) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call(5), Verdict::Reject(()));
    assert_eq!(f.call(10), Verdict::Accept(()));
}

#[test]
fn equal_to_10_with_function() {
    let mut p = compile(src! {"
        fn is_10(x: i32) -> bool {
            x == 10
        }

        filtermap main(x: i32) {
            if is_10(x) { accept } else { reject }
        }
    "});
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call(5), Verdict::Reject(()));
    assert_eq!(f.call(10), Verdict::Accept(()));
}

#[test]
fn equal_to_10_with_two_functions() {
    let mut p = compile(src! {"
        fn equals(x: u32, y: u32) -> bool {
            x == y
        }

        fn is_10(x: u32) -> bool {
            equals(x, 10)
        }

        filtermap main(x: u32) {
            if is_10(x) {
                accept
            } else if equals(x, 20) {
                accept
            } else {
                reject
            }
        }
    "});
    let f = p.func::<fn(u32) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call(5), Verdict::Reject(()));
    assert_eq!(f.call(10), Verdict::Accept(()));
    assert_eq!(f.call(15), Verdict::Reject(()));
    assert_eq!(f.call(20), Verdict::Accept(()));
}

#[test]
fn negation() {
    let mut p = compile(src!("fn negate(x: i32) -> i32 { -x }"));
    let f = p.func::<fn(i32) -> i32>("negate").unwrap();
    assert_eq!(f.call(5), -5);

    let mut p = compile(src!("fn negate() -> i32 { -5 }"));
    let f = p.func::<fn() -> i32>("negate").unwrap();
    assert_eq!(f.call(), -5);
}

#[test]
fn inversion() {
    let mut p = compile(src! {"
        filtermap main(x: i32) {
            if not (x == 10) {
                accept
            } else {
                reject
            }
        }
    "});
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    for x in 0..20 {
        #[allow(clippy::if_not_else, clippy::nonminimal_bool)]
        let exp = if !(x == 10) {
            Verdict::Accept(())
        } else {
            Verdict::Reject(())
        };
        assert_eq!(f.call(x), exp, "{x}");
    }
}

#[test]
fn not_not() {
    let mut p = compile(src! {"
        fn main(x: i32) -> bool {
            not not (x == 10)
        }
    "});
    let f = p.func::<fn(i32) -> bool>("main").unwrap();

    for x in 0..20 {
        assert_eq!(f.call(x), x == 10, "{x}");
    }
}

#[test]
fn a_bunch_of_comparisons() {
    let mut p = compile(src! {"
        filtermap main(x: i32) {
            if (
                (x > 10 && x < 20)
                || (x >= 30 && x <= 40)
                || x == 55
            ){
                accept
            } else {
                reject
            }
        }
    "});
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    for x in 0..100 {
        #[allow(clippy::manual_range_contains)]
        let expected = if (x > 10 && x < 20) || (x >= 30 && x <= 40) || x == 55 {
            Verdict::Accept(())
        } else {
            Verdict::Reject(())
        };

        assert_eq!(f.call(x), expected);
    }
}

#[test]
fn record() {
    let s = src!(
        "
        record Foo { a: i32, b: i32 }

        filtermap main(x: i32) {
            let foo = Foo { a: x, b: 20 };
            if foo.a == foo.b {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    for x in 0..100 {
        let expected = if x == 20 {
            Verdict::Accept(())
        } else {
            Verdict::Reject(())
        };
        assert_eq!(f.call(x), expected);
    }
}

#[test]
fn record_with_fields_flipped() {
    let s = src!(
        "
        record Foo { a: i32, b: i32 }

        filtermap main(x: i32) {
            # These are flipped, to ensure that the order in which
            # the fields are given doesn't matter:
            let foo = Foo { b: 20, a: x };
            if foo.a == foo.b {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    for x in 0..100 {
        let expected = if x == 20 {
            Verdict::Accept(())
        } else {
            Verdict::Reject(())
        };
        assert_eq!(f.call(x), expected);
    }
}

#[test]
fn nested_record() {
    let s = src!(
        "
        record Foo { x: Bar, y: Bar }
        record Bar { a: i32, b: i32 }

        filtermap main(x: i32) {
            let bar = Bar { a: 20, b: x };
            let foo = Foo { x: bar, y: bar };
            if foo.x.a == foo.y.b {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    for x in 0..100 {
        let expected = if x == 20 {
            Verdict::Accept(())
        } else {
            Verdict::Reject(())
        };
        assert_eq!(f.call(x), expected, "for {x}");
    }
}

#[test]
fn misaligned_fields() {
    // A record where the second field should be aligned
    let s = src!(
        "
        record Foo { a: i16, b: i32 }

        filtermap main(x: i32) {
            let foo = Foo { a: 10, b: x };
            if foo.b == 20 {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    for x in 0..100 {
        let expected = if x == 20 {
            Verdict::Accept(())
        } else {
            Verdict::Reject(())
        };
        assert_eq!(f.call(x), expected, "for {x}");
    }
}

#[test]
fn arithmetic() {
    let s = src!(
        "
        filtermap main(x: i32) {
            if x + 10 * 20 < 250 {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call(5), Verdict::Accept(()));
    assert_eq!(f.call(20), Verdict::Accept(()));
    assert_eq!(f.call(100), Verdict::Reject(()));
}

#[test]
fn call_runtime_function() {
    let s = src!(
        "
        filtermap main(x: u32) {
            if pow(x, 2) > 100 {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut lib = Library::default();
    lib.define_func("pow", "", ["x", "y"], |x: u32, y: u32| -> u32 { x.pow(y) });

    let rt = Runtime::from_lib(lib).unwrap();

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn(u32) -> Verdict<(), ()>>("main").unwrap();

    for (value, expected) in [(5, Verdict::Reject(())), (11, Verdict::Accept(()))] {
        assert_eq!(f.call(value), expected);
    }
}

#[test]
fn call_runtime_method() {
    let s = src!(
        "
        filtermap main(x: u32) {
            if x.is_even() {
                accept
            } else {
                reject
            }
        }
    "
    );

    let mut impl_block = roto::Impl::new::<u32>();
    impl_block.define_func("is_even", "", ["self"], |this: u32| -> bool {
        this.is_multiple_of(2)
    });
    let lib = Library::default().with(impl_block);

    let rt = Runtime::from_lib(lib).unwrap();

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn(u32) -> Verdict<(), ()>>("main").unwrap();

    for (value, expected) in [(5, Verdict::Reject(())), (10, Verdict::Accept(()))] {
        assert_eq!(f.call(value), expected);
    }
}

#[test]
fn int_var() {
    let s = src!(
        "
        filtermap main() {
            accept 32
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Verdict<i32, ()>>("main").unwrap();

    assert_eq!(f.call(), Verdict::Accept(32));
}

#[test]
fn issue_52() {
    #[derive(Clone)]
    struct Foo {
        _x: i32,
    }

    let mut impl_block = roto::Impl::new::<Val<Foo>>();
    impl_block.define_func("bar", "", ["_x"], |_x: u32| -> u32 { 2 });

    let rt = Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_clone_ty::<Val<Foo>>("Foo", "A Foo!");
        lib.add(impl_block.into());
        lib
    })
    .unwrap();

    let s = src!("filtermap main(foo: Foo) { Foo.bar(1); accept }");

    let _p = compile_with_runtime(s, rt);
}

#[test]
fn register_with_non_registered_type() {
    #[derive(Clone)]
    struct Foo {
        _x: i32,
    }

    let mut lib = Library::default();
    lib.define_func("bar", "", ["_foo", "_x"], |_foo: Val<Foo>, _x: u32| {});

    Runtime::from_lib(lib).unwrap_err();
}

#[test]
fn mismatched_types() {
    let s = src!(
        "
        filtermap main(x: i32) {
            accept x
        }
    "
    );

    let mut p = compile(s);

    let err = p.func::<fn(i8) -> Verdict<i8, ()>>("main").unwrap_err();

    eprintln!("{err}");
    assert!(err.to_string().contains("do not match"));
}

#[test]
fn multiply() {
    let s = src!(
        "
        filtermap main(x: u8) {
            if x > 10 {
                accept 2 * x
            } else {
                reject
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(u8) -> Verdict<u8, ()>>("main").unwrap();

    assert_eq!(f.call(20), Verdict::Accept(40));
}

#[test]
fn remainder() {
    let s = src!(
        "
        fn remainder(x: u64, y: u64) -> u64 {
            while x > y {
                x = x - y;
            }
            x
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(u64, u64) -> u64>("remainder").unwrap();

    assert_eq!(f.call(55, 10), 5);
}

#[test]
fn factorial() {
    let s = src!(
        "
        fn factorial(x: u64) -> u64 {
            let i = 1;
            let n = 1;
            while i <= x {
                n = n * i;
                i = i + 1;
            }
            n
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(u64) -> u64>("factorial").unwrap();

    assert_eq!(f.call(5), 120);
}

#[test]
fn nested_while_loop() {
    let s = src!(
        "
        fn nested_while_loop() -> u64 {
            let res = 0;
            let x = 0;
            while x < 10 {
                let y = 0;
                while y < 10 {
                    res = res + 1;
                    y = y + 1;
                }
                x = x + 1;
            }
            res
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> u64>("nested_while_loop").unwrap();

    assert_eq!(f.call(), 100);
}

#[test]
fn repeat_string_manually() {
    let s = src!(
        "
        fn repeat(s: String, n: u64) -> String {
            let res = \"\";
            while n > 0 {
                res = res + s;
                n = n - 1;
            }
            res
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(Arc<str>, u64) -> Arc<str>>("repeat").unwrap();

    assert_eq!(f.call("foo".into(), 6), "foofoofoofoofoofoo".into());
}

#[test]
fn float_mul() {
    let s = src!(
        "
        filtermap main(x: f32) {
            accept 2.0 * x
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(f32) -> Verdict<f32, ()>>("main").unwrap();

    assert_eq!(f.call(20.0), Verdict::Accept(40.0));
}

#[test]
fn float_add() {
    let s = src!(
        "
        filtermap main(x: f32) {
            accept 2.0 + x
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(f32) -> Verdict<f32, ()>>("main").unwrap();

    assert_eq!(f.call(20.0), Verdict::Accept(22.0));
}

#[test]
fn float_sub() {
    let s = src!(
        "
        filtermap main(x: f32) {
            accept 20.0 - x
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(f32) -> Verdict<f32, ()>>("main").unwrap();

    assert_eq!(f.call(2.0), Verdict::Accept(18.0));
}

#[test]
fn float_cmp() {
    let mut p = compile(src!("filtermap main(x: f32) { accept x == 20.0}"));
    let f = p.func::<fn(f32) -> Verdict<bool, ()>>("main").unwrap();
    assert_eq!(f.call(20.0), Verdict::Accept(true));
}

#[test]
fn float_div_zero() {
    let mut p = compile(src!("filtermap main(x: f32) { accept x / 0.0 }"));
    let f = p.func::<fn(f32) -> Verdict<f32, ()>>("main").unwrap();

    assert_eq!(f.call(20.0), Verdict::Accept(f32::INFINITY));
    assert_eq!(f.call(-20.0), Verdict::Accept(-f32::INFINITY));

    let Verdict::Accept(res) = f.call(0.0) else {
        panic!("should have returned accept")
    };
    assert!(res.is_nan());
}

#[test]
fn float_floor() {
    let s = src!("fn floor(x: f32) -> f32 { x.floor() }");
    let mut p = compile(s);
    let f = p.func::<fn(f32) -> f32>("floor").unwrap();

    assert_eq!(f.call(20.5), 20.0);
    assert_eq!(f.call(-20.5), -21.0);
}

#[test]
fn float_ceil() {
    let s = src!("fn ceil(x: f32) -> f32 { x.ceil() }");
    let mut p = compile(s);
    let f = p.func::<fn(f32) -> f32>("ceil").unwrap();

    assert_eq!(f.call(20.5), 21.0);
    assert_eq!(f.call(-20.5), -20.0);
}

#[test]
fn float_round() {
    let s = src!("fn round(x: f32) -> f32 { x.round() }");
    let mut p = compile(s);
    let f = p.func::<fn(f32) -> f32>("round").unwrap();

    assert_eq!(f.call(20.6), 21.0);
    assert_eq!(f.call(20.4), 20.0);
}

#[test]
fn float_pow() {
    let s = src!("fn pow(x: f32, y: f32) -> f32 { x.pow(y) }");
    let mut p = compile(s);
    let f = p.func::<fn(f32, f32) -> f32>("pow").unwrap();

    assert_eq!(f.call(2.0, 2.0), 4.0);
    assert_eq!(f.call(25.0, 0.5), 5.0);
}

#[test]
fn float_scientific_notation_one() {
    let s = src!("fn main() -> f32 { 20.0e4 }");

    let mut p = compile(s);
    let f = p.func::<fn() -> f32>("main").unwrap();

    assert_eq!(f.call(), 20.0e4);
}

#[test]
fn float_scientific_notation_two() {
    let s = src!("fn main() -> f32 { 20.0e-4 }");

    let mut p = compile(s);
    let f = p.func::<fn() -> f32>("main").unwrap();

    assert_eq!(f.call(), 20.0e-4);
}

#[test]
fn float_add_f64() {
    let s = src!(
        "
        filtermap main(x: f64) {
            accept x + 20.0
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(f64) -> Verdict<f64, ()>>("main").unwrap();

    assert_eq!(f.call(20.0), Verdict::Accept(40.0));
}

#[test]
fn function_returning_unit() {
    let mut lib = Library::default();
    lib.define_func("unit_unit", "", [], || ());
    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!("filtermap main() { accept unit_unit() }");

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn() -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call(), Verdict::Accept(()));
}

#[test]
fn to_string() {
    let s = src!(
        r#"
        fn foo() -> String {
            let a: u8 = 10;
            let b: i32 = 20;
            let c: f64 = 15.5;
            let d = false;
            let h = "foo";

              a.to_string() + " "
            + b.to_string() + " "
            + c.to_string() + " "
            + d.to_string() + " "
            + h.to_string()
        }
    "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(), "10 20 15.5 false foo".into());
}

#[test]
fn simple_f_string() {
    let s = src!(
        r#"
        fn foo(name: String) -> String {
            f"Hello {name}!"
        }
    "#
    );

    let mut p = compile(s);
    let f = p.func::<fn(Arc<str>) -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call("John".into()), "Hello John!".into());
}

#[test]
fn simple_f_string_number() {
    let s = src!(
        r#"
        fn foo(x: i32) -> String {
            f"Hello {x}!"
        }
    "#
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(10), "Hello 10!".into());
}

#[test]
fn complex_f_string() {
    let s = src!(
        r#"
        fn foo() -> String {
            f"This is a string with { f"another string which prints {true}" }, isn't that wonderful?"
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(
        f.call(),
        "This is a string with another string which prints true, isn't that wonderful?".into()
    );
}

#[test]
fn escape_curly_in_f_string() {
    let s = src!(
        r#"
        fn foo() -> String {
            f"Here is a single curly {{ and a closing one }}"
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(
        f.call(),
        "Here is a single curly { and a closing one }".into()
    );
}

#[test]
fn unicode_val_in_f_string() {
    let s = src!(
        r#"
        fn foo() -> String {
            f"Here is an uppercase \u{41} and a lowercase \u{61}."
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(), "Here is an uppercase A and a lowercase a.".into());
}

#[test]
fn f_string_and_int_var_1() {
    let s = src!(
        r#"
        fn foo() -> String {
            let x = 10;
            f"This should just work: {x}"
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(), "This should just work: 10".into());
}

#[test]
fn f_string_and_int_var_2() {
    let s = src!(
        r#"
        fn foo() -> String {
            let x = 10;
            let s = f"This should just work: {x}";
            let y: u8 = x;
            s
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(), "This should just work: 10".into());
}

#[test]
fn f_string_and_float_var_1() {
    let s = src!(
        r#"
        fn foo() -> String {
            let x = 10.0;
            f"This should just work: {x}"
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(), "This should just work: 10".into());
}

#[test]
fn f_string_and_float_var_2() {
    let s = src!(
        r#"
        fn foo() -> String {
            let x = 10.0;
            let s = f"This should just work: {x}";
            let y: f32 = x;
            s
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(), "This should just work: 10".into());
}

#[test]
fn arc_type() {
    use std::sync::atomic::Ordering;

    static CLONES: AtomicUsize = AtomicUsize::new(0);
    static DROPS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct CloneDrop {
        clones: &'static AtomicUsize,
        drops: &'static AtomicUsize,
    }

    impl Clone for CloneDrop {
        fn clone(&self) -> Self {
            self.clones.fetch_add(1, Ordering::Relaxed);
            Self {
                clones: self.clones,
                drops: self.drops,
            }
        }
    }

    impl Drop for CloneDrop {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    let mut lib = Library::default();
    lib.define_clone_ty::<Val<CloneDrop>>(
        "CloneDrop",
        "A special type for which we track the number of clones and drops",
    );

    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!(
        "
        filtermap main(choose: bool, x: CloneDrop, y: CloneDrop) {
            if choose {
                accept x
            } else {
                accept y
            }
        }
    "
    );

    let mut p = compile_with_runtime(s, rt);
    let f = p
        .func::<fn(bool, Val<CloneDrop>, Val<CloneDrop>) -> Verdict<Val<CloneDrop>, ()>>("main")
        .unwrap();

    let input = CloneDrop {
        clones: &CLONES,
        drops: &DROPS,
    };

    println!("{input:?}");

    let output = f.call(true, Val(input.clone()), Val(input));

    let output = output.into_result().unwrap().0;
    println!("{output:?}");
    assert_eq!(
        output.clones.load(Ordering::Relaxed),
        output.drops.load(Ordering::Relaxed)
    );
}

#[test]
fn registered_constant() {
    let mut lib = roto::Library::default();
    lib.define_const("PI", "", std::f64::consts::PI);
    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!("fn circumference(radius: f64) -> f64 { 2.0 * PI * radius }");

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn(f64) -> f64>("circumference").unwrap();
    let output = f.call(2.0);

    let exp = 2.0 * std::f64::consts::PI * 2.0;
    assert!(exp - 0.1 < output && output < exp + 0.1);
}

#[test]
fn use_a_roto_function() {
    let s = src!(
        "
        fn double(x: i32) -> i32 {
            2 * x
        }"
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32) -> i32>("double").unwrap();
    let output = f.call(2);
    assert_eq!(output, 4);

    let output = f.call(16);
    assert_eq!(output, 32);
}

#[test]
fn use_a_test() {
    let s = src!(
        "
        fn double(x: i32) -> i32 {
            x # oops! not correct
        }

        test check_double {
            if double(4) != 8 {
                reject;
            }
            if double(16) != 32 {
                reject;
            }
            accept
        }
        "
    );

    let mut p = compile(s);
    p.run_tests().unwrap_err();

    let s = src!(
        "
        fn double(x: i32) -> i32 {
            2 * x
        }

        test check_double {
            if double(4) != 8 {
                reject;
            }
            if double(16) != 32 {
                reject;
            }
            accept
        }
        "
    );

    let mut p = compile(s);
    p.run_tests().unwrap();
}

#[test]
fn get_tests() {
    let s = src!(
        "
        test check_output {
            accept
        }
        "
    );

    let mut p = compile(s);
    let tests: Vec<_> = p.tests().collect();

    assert!(!tests.is_empty());
    assert_eq!(tests[0].name(), "pkg.check_output");
    tests[0].run().unwrap();
}

#[test]
fn get_no_tests() {
    let s = src!(
        "
        fn double(x: i32) -> i32 {
          2 * x
        }
        "
    );

    let mut p = compile(s);
    let tests = p.tests();
    assert_eq!(tests.count(), 0);
}

#[test]
fn string() {
    let s = src!(
        r#"
        filtermap main() {
            accept "hello"
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn() -> Verdict<Arc<str>, ()>>("main").unwrap();

    assert_eq!(f.call(), Verdict::Accept("hello".into()));
}

#[test]
fn escape_string() {
    let s = src!(
        r#"
        fn main() -> String {
            "\t\tfoo"
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn() -> Arc<str>>("main").unwrap();

    assert_eq!(f.call(), "\t\tfoo".into());
}

#[test]
fn string_append() {
    let s = src!(
        r#"
        filtermap main(name: String) {
            accept "Hello ".append(name).append("!")
        }
    "#
    );

    let mut p = compile(s);

    let f = p
        .func::<fn(Arc<str>) -> Verdict<Arc<str>, ()>>("main")
        .unwrap();

    assert_eq!(
        f.call("Martin".into()),
        Verdict::Accept("Hello Martin!".into())
    );
}

#[test]
fn string_append_as_function_call() {
    let s = src!(
        r#"
        fn main(name: String) -> String {
            String.append("Hello ", name)
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn(Arc<str>) -> Arc<str>>("main").unwrap();

    assert_eq!(f.call("Martin".into()), "Hello Martin".into());
}

#[test]
fn string_append_as_imported_function() {
    let s = src!(
        r#"
        import String.append;

        fn main(name: String) -> String {
            append("Hello ", name)
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn(Arc<str>) -> Arc<str>>("main").unwrap();

    assert_eq!(f.call("Martin".into()), "Hello Martin".into());
}

#[test]
fn string_plus_operator() {
    let s = src!(
        r#"
        filtermap main(name: String) {
            accept "Hello " + name + "!"
        }
    "#
    );

    let mut p = compile(s);

    let f = p
        .func::<fn(Arc<str>) -> Verdict<Arc<str>, ()>>("main")
        .unwrap();

    assert_eq!(
        f.call("Martin".into()),
        Verdict::Accept("Hello Martin!".into())
    );
}

#[test]
fn string_contains() {
    let s = src!(
        r#"
        filtermap main(s: String) {
            if "incomprehensibilities".contains(s) {
                accept
            } else {
                reject
            }
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn(Arc<str>) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call("incompre".into()), Verdict::Accept(()));
    assert_eq!(f.call("hensi".into()), Verdict::Accept(()));
    assert_eq!(f.call("bilities".into()), Verdict::Accept(()));
    assert_eq!(f.call("nananana".into()), Verdict::Reject(()));
}

#[test]
fn string_starts_with() {
    let s = src!(
        r#"
        filtermap main(s: String) {
            if "incomprehensibilities".starts_with(s) {
                accept
            } else {
                reject
            }
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn(Arc<str>) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call("incompre".into()), Verdict::Accept(()));
    assert_eq!(f.call("hensi".into()), Verdict::Reject(()));
    assert_eq!(f.call("bilities".into()), Verdict::Reject(()));
    assert_eq!(f.call("nananana".into()), Verdict::Reject(()));
}

#[test]
fn string_ends_with() {
    let s = src!(
        r#"
        filtermap main(s: String) {
            if "incomprehensibilities".ends_with(s) {
                accept
            } else {
                reject
            }
        }
    "#
    );

    let mut p = compile(s);

    let f = p.func::<fn(Arc<str>) -> Verdict<(), ()>>("main").unwrap();

    assert_eq!(f.call("incompre".into()), Verdict::Reject(()));
    assert_eq!(f.call("hensi".into()), Verdict::Reject(()));
    assert_eq!(f.call("bilities".into()), Verdict::Accept(()));
    assert_eq!(f.call("nananana".into()), Verdict::Reject(()));
}

#[test]
fn string_to_lowercase_and_uppercase() {
    let s = src!(
        r"
        filtermap main(lower: bool, s: String) {
            if lower {
                accept s.to_lowercase()
            } else {
                accept s.to_uppercase()
            }
        }
    "
    );

    let mut p = compile(s);

    let f = p
        .func::<fn(bool, Arc<str>) -> Verdict<Arc<str>, ()>>("main")
        .unwrap();

    assert_eq!(
        f.call(true, "WHISPER THIS!".into()),
        Verdict::Accept("whisper this!".into())
    );
    assert_eq!(
        f.call(false, "now shout this!".into()),
        Verdict::Accept("NOW SHOUT THIS!".into())
    );
}

#[test]
fn string_repeat() {
    let s = src!(
        r#"
        filtermap main(s: String) {
            let exclamation = (s + "!").to_uppercase();
            accept (exclamation + " ").repeat(4) + exclamation
        }
    "#
    );

    let mut p = compile(s);

    let f = p
        .func::<fn(Arc<str>) -> Verdict<Arc<str>, ()>>("main")
        .unwrap();

    assert_eq!(
        f.call("boo".into()),
        Verdict::Accept("BOO! BOO! BOO! BOO! BOO!".into())
    );
}

#[test]
fn match_option_value() {
    let s = src!(
        "
        fn or_fortytwo(x: u32?) -> u32 {
            match x {
                Some(x) -> x,
                None -> 42,
            }
        }
        "
    );

    let mut p = compile(s);

    let f = p.func::<fn(Option<u32>) -> u32>("or_fortytwo").unwrap();

    assert_eq!(f.call(Some(10)), 10);
    assert_eq!(f.call(None), 42);
}

#[test]
fn match_option_string() {
    let s = src!(
        "
        fn foo() -> String? {
            Option.Some(\"Foobar\")
        }

        fn bar() -> Verdict[String, String] {
            match foo() {
                Some(v) -> accept v,
                _ -> reject \"nope\",
            }
        }
        "
    );

    let mut p = compile(s);

    let f = p
        .func::<fn() -> Verdict<Arc<str>, Arc<str>>>("bar")
        .unwrap();

    assert_eq!(f.call(), Verdict::Accept("Foobar".into()));
}

#[test]
fn match_on_string_with_guards() {
    let s = src!(
        r#"
        fn foo(s: String?, x: i32) -> String {
            match s {
                Some(s) | s == "hello" -> "hey!",
                _ | x == 5 ->  "x is 5",
                Some(s) | s.starts_with("lorem ipsum") -> {
                    "You've generated lorem ipsum: " + s
                }
                _ -> "Can't recognize",
            }
        }
        "#
    );

    let mut p = compile(s);

    let f = p
        .func::<fn(Option<Arc<str>>, i32) -> Arc<str>>("foo")
        .unwrap();

    assert_eq!(&*f.call(Some("hello".into()), 5), "hey!");
    assert_eq!(&*f.call(Some("hello".into()), 4), "hey!");
    assert_eq!(&*f.call(Some("lorem ipsum dolor".into()), 5), "x is 5");
    assert_eq!(
        &*f.call(Some("lorem ipsum dolor".into()), 4),
        "You've generated lorem ipsum: lorem ipsum dolor"
    );
    assert_eq!(&*f.call(Some("nothing".into()), 5), "x is 5");
    assert_eq!(&*f.call(Some("nothing".into()), 4), "Can't recognize");
}

#[test]
fn construct_option_value() {
    let s = src!(
        "
        fn sub_one(x: u32) -> u32? {
            if x == 0 {
                Option.None
            } else {
                Option.Some(x - 1)
            }
        }
        "
    );

    let mut p = compile(s);

    let f = p.func::<fn(u32) -> Option<u32>>("sub_one").unwrap();

    assert_eq!(f.call(0), None);
    assert_eq!(f.call(2), Some(1));
}

#[test]
fn construct_imported_option() {
    let s = src!(
        "
        import Option.{Some, None};

        fn sub_one(x: u32) -> u32? {
            if x == 0 {
                None
            } else {
                Some(x - 1)
            }
        }
        "
    );

    let mut p = compile(s);

    let f = p.func::<fn(u32) -> Option<u32>>("sub_one").unwrap();

    assert_eq!(f.call(0), None);
    assert_eq!(f.call(2), Some(1));
}

#[test]
fn construct_option_value_from_prelude() {
    let s = src!(
        "
        fn sub_one(x: u32) -> u32? {
            if x == 0 {
                None
            } else {
                Some(x - 1)
            }
        }
        "
    );

    let mut p = compile(s);

    let f = p.func::<fn(u32) -> Option<u32>>("sub_one").unwrap();

    assert_eq!(f.call(0), None);
    assert_eq!(f.call(2), Some(1));
}

#[test]
fn filter_map_with_manual_verdict() {
    let s = src!(
        "
        filtermap foo(x: i32) {
            if x < 10 {
                return Verdict.Reject(10)
            } else {
                return Verdict.Accept(x)
            }
        }
        "
    );

    let mut p = compile(s);

    let f = p.func::<fn(i32) -> Verdict<i32, i32>>("foo").unwrap();

    assert_eq!(f.call(0), Verdict::Reject(10));
    assert_eq!(f.call(12), Verdict::Accept(12));
}

/// This tests the type checker because it needs to infer a never type for Reject
#[test]
fn unused_accept() {
    let mut p = compile(src!("fn foo() { Verdict.Accept(5); }"));
    p.func::<fn() -> ()>("foo").unwrap().call();
}

#[test]
fn match_on_verdict() {
    let s = src!(
        "
        filtermap over_10(x: i32) {
            if x > 10 {
                accept x
            } else {
                reject x
            }
        }

        filtermap foo(x: i32) {
            match over_10(x) {
                Accept(x) -> accept x + 1,
                Reject(x) -> reject x,
            };
        }
        "
    );

    let mut p = compile(s);

    let f = p.func::<fn(i32) -> Verdict<i32, i32>>("foo").unwrap();

    assert_eq!(f.call(5), Verdict::Reject(5));
    assert_eq!(f.call(15), Verdict::Accept(16));
    assert_eq!(f.call(25), Verdict::Accept(26));
}

#[test]
fn non_sugar_option() {
    let mut p = compile(src!("fn foo() -> Option[u32] { Option.Some(2) }"));
    let f = p.func::<fn() -> Option<u32>>("foo").unwrap();
    assert_eq!(f.call(), Some(2));
}

#[test]
fn none_with_unknown_type() {
    let mut p = compile(src!("fn foo() { Option.None; }"));
    p.func::<fn() -> ()>("foo").unwrap().call();
}

#[test]
fn question_mark() {
    let s = src!(
        "
        fn bar() -> u32? {
            Option.None
        }

        fn foo() -> u32? {
            bar()?;
            Option.Some(3)
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Option<u32>>("foo").unwrap();
    assert_eq!(f.call(), None);
}

#[test]
fn question_mark_in_chain() {
    let s = src!(
        "
        fn is_finite(x: f64) -> f64? {
            if x.is_finite() {
                Option.Some(x)
            } else {
                Option.None
            }
        }

        fn foo(x: f64) -> f64? {
            let y = is_finite(x)?.pow(2.0);
            Option.Some(y)
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(f64) -> Option<f64>>("foo").unwrap();

    assert_eq!(f.call(10.0), Some(100.0));
    assert_eq!(f.call(f64::NAN), None);
}

#[test]
fn question_unit() {
    let s = src!(
        "
        fn maybe_a_unit(x: bool) -> ()? {
            if x {
                Some(())
            } else {
                None
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(bool) -> Option<()>>("maybe_a_unit").unwrap();

    assert_eq!(f.call(false), None);
    assert_eq!(f.call(true), Some(()));
}

#[test]
fn question_record() {
    let s = src!(
        "
        fn maybe_an_i32(x: bool) -> i32? {
            Some(maybe_a_record(x)?.a)
        }

        fn maybe_a_record(x: bool) -> { a: i32 }? {
            if x {
                Some({ a: 5 })
            } else {
                None
            }
        }
    "
    );

    let mut p = compile(s);
    let f = p.func::<fn(bool) -> Option<i32>>("maybe_an_i32").unwrap();

    assert_eq!(f.call(false), None);
    assert_eq!(f.call(true), Some(5));
}

#[test]
fn add_options() {
    let s = src!(
        "
        fn small(x: i32) -> i32? {
            if x < 10 {
                Option.Some(x)
            } else {
                Option.None
            }
        }

        fn foo(x: i32, y: i32) -> i32? {
            let z = small(x)? + small(y)?;
            Option.Some(z)
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(i32, i32) -> Option<i32>>("foo").unwrap();

    assert_eq!(f.call(2, 2), Some(4));
    assert_eq!(f.call(5, 5), Some(10));
    assert_eq!(f.call(15, 5), None);
    assert_eq!(f.call(5, 10), None);
}

#[test]
fn question_question() {
    let s = src!(
        "
        fn bar() -> u32?? {
            Option.Some(Option.None)
        }

        fn foo() -> u32? {
            bar()??;
            Option.Some(4)
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Option<u32>>("foo").unwrap();
    assert_eq!(f.call(), None);
}

#[test]
fn question_mark_none() {
    let s = src!(
        "
        fn foo() -> u32? {
            Option.None?;
            Option.Some(3)
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn() -> Option<u32>>("foo").unwrap();
    assert_eq!(f.call(), None);
}

#[test]
fn top_level_import() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.bar;
            fn main(x: i32) -> i32 {
                bar(x)
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            fn bar(x: i32) -> i32 {
                2 * x
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 8);
}

#[test]
fn local_import() {
    let pkg = source_file!(
        "pkg",
        "
            fn main(x: i32) -> i32 {
                import foo.bar;
                bar(x)
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            fn bar(x: i32) -> i32 {
                2 * x
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 8);
}

#[test]
fn parent_import() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.quadruple;
            fn main(x: i32) -> i32 {
                quadruple(x)
            }

            fn double(x: i32) -> i32 {
                2 * x
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            import super.double;
            fn quadruple(x: i32) -> i32 {
                double(double(x))
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 16);
}

#[test]
fn package_import() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.quadruple;
            fn main(x: i32) -> i32 {
                quadruple(x)
            }

            fn double(x: i32) -> i32 {
                2 * x
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            import pkg.double;
            fn quadruple(x: i32) -> i32 {
                double(double(x))
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 16);
}

#[test]
fn import_via_super() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.a;
            fn main(x: i32) -> i32 {
                a(x)
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            import super.bar.b;
            fn a(x: i32) -> i32 {
                b(x)
            }
        "
    );
    let bar = source_file!(
        "bar",
        "
            fn b(x: i32) -> i32 {
                2 * x
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(
        pkg,
        vec![FileSpec::File(foo), FileSpec::File(bar)],
    ));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 8);
}

#[test]
fn import_module_first() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.a;
            fn main(x: i32) -> i32 {
                a(x)
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            import super.bar;
            import bar.b;
            fn a(x: i32) -> i32 {
                b(x)
            }
        "
    );
    let bar = source_file!(
        "bar",
        "
            fn b(x: i32) -> i32 {
                2 * x
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(
        pkg,
        vec![FileSpec::File(foo), FileSpec::File(bar)],
    ));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 8);
}

#[test]
fn import_module_second() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.a;
            fn main(x: i32) -> i32 {
                a(x)
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            import bar.b;
            import super.bar;
            fn a(x: i32) -> i32 {
                b(x)
            }
        "
    );
    let bar = source_file!(
        "bar",
        "
            fn b(x: i32) -> i32 {
                2 * x
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(
        pkg,
        vec![FileSpec::File(foo), FileSpec::File(bar)],
    ));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 8);
}

#[test]
fn use_type_from_module() {
    let pkg = source_file!(
        "pkg",
        "
            fn main(x: i32) -> i32 {
                let foofoo = foo.Foo { bar: x };
                foofoo.bar
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            record Foo {
                bar: i32,
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 4);
}

#[test]
fn use_imported_type() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.Foo;
            fn main(x: i32) -> i32 {
                let foofoo = Foo { bar: x };
                foofoo.bar
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            record Foo {
                bar: i32,
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 4);
}

#[test]
fn use_type_in_function_argument() {
    let pkg = source_file!(
        "pkg",
        "
            fn main(x: i32) -> i32 {
                get_bar(foo.Foo { bar: x })
            }

            fn get_bar(f: foo.Foo) -> i32 {
                f.bar
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            record Foo {
                bar: i32,
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 4);
}

#[test]
fn import_list() {
    let pkg = source_file!(
        "pkg",
        "
            import foo.{double, triple};
            fn main(x: i32) -> i32 {
                triple(double(x))
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            fn double(x: i32) -> i32 {
                2 * x
            }

            fn triple(x: i32) -> i32 {
                3 * x
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(1), 6);
}

#[test]
fn use_type_in_function_return_type() {
    let pkg = source_file!(
        "pkg",
        "
            fn main(x: i32) -> i32 {
                make_foo(x).bar
            }

            fn make_foo(x: i32) -> foo.Foo {
                foo.Foo { bar: x }
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            record Foo {
                bar: i32,
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 4);
}

#[test]
fn use_type_from_other_module_in_type() {
    let pkg = source_file!(
        "pkg",
        "
            record Bli {
                bla: foo.Bla,
            }

            fn main(x: i32) -> i32 {
                Bli { bla: foo.Bla { blubb: x }}.bla.blubb
            }
        "
    );
    let foo = source_file!(
        "foo",
        "
            record Bla {
                blubb: i32,
            }
        "
    );

    let tree = FileTree::file_spec(FileSpec::Directory(pkg, vec![FileSpec::File(foo)]));
    let mut p = compile(tree);
    let main = p.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(main.call(4), 4);
}

#[test]
fn mutate() {
    #[derive(Copy, Clone, Debug)]
    struct MyType {
        i: i16,
    }

    let mut impl_block = roto::Impl::new::<Val<*mut MyType>>();
    impl_block.define_func("increase", "", ["self"], |mut this: Val<*mut MyType>| {
        eprintln!("increase, pre: {}", unsafe { (**this).i });
        unsafe { (**this).i += 1 };
        eprintln!("increase, post: {}", unsafe { (**this).i });
    });

    let rt = Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_copy_ty::<Val<*mut MyType>>("MyType", "");
        lib.add(impl_block.into());
        lib
    })
    .unwrap();

    let s = src!("filter main(t: MyType) { t.increase(); accept t; }");

    let mut p = compile_with_runtime(s, rt);
    let f = p
        .func::<fn(Val<*mut MyType>) -> Verdict<Val<*mut MyType>, ()>>("main")
        .unwrap();

    let mut t = MyType { i: 0 };
    match f.call(Val(&raw mut t)) {
        Verdict::Accept(val) => {
            let val = unsafe { &*val.0 };
            assert_eq!(val.i, 1, "returned value should be 1");
        }
        Verdict::Reject(()) => todo!(),
    }
    assert_eq!(t.i, 1, "mutated value should be 1");
}

#[test]
fn return_vec() {
    #[derive(Clone, Debug, Default)]
    #[allow(dead_code)]
    struct MyType {
        v: Vec<u8>,
    }

    let mut lib = Library::default();
    lib.define_clone_ty::<Val<MyType>>("MyType", "");
    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!("fn main(t: MyType) -> MyType { t }");

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn(Val<MyType>) -> Val<MyType>>("main").unwrap();

    let t = MyType { v: vec![0x01] };
    println!("Returned with {:?}", f.call(Val(t)).0.v.as_ptr());
}

#[test]
fn name_collision() {
    #[derive(Clone)]
    struct A {
        _x: u32,
    }

    #[derive(Clone)]
    struct B {
        _x: u32,
    }

    for _ in 0..100 {
        let mut a_impl = roto::Impl::new::<Val<A>>();
        a_impl.define_func("foo", "", ["self"], |_: Val<A>| -> bool { true });
        let mut b_impl = roto::Impl::new::<Val<B>>();
        b_impl.define_func("foo", "", ["self"], |_: Val<B>| -> bool { unreachable!() });

        let rt = Runtime::from_lib({
            let mut lib = roto::Library::default();
            lib.define_clone_ty::<Val<A>>("A", "");
            lib.define_clone_ty::<Val<B>>("B", "");
            lib.add(a_impl.into());
            lib.add(b_impl.into());
            lib
        })
        .unwrap();

        let s = src!(
            "
           filter foo(t: B) {
               accept t.foo()
           }

           filter main(t: A) {
               accept t.foo()
           }
       "
        );

        let mut p = compile_with_runtime(s, rt);
        let f = p.func::<fn(Val<A>) -> Verdict<bool, ()>>("main").unwrap();

        let t = A { _x: 1 };
        assert_eq!(f.call(Val(t)), Verdict::Accept(true));
        eprint!(".");
    }
}

#[test]
fn refcounting_in_a_recursive_function() {
    #[derive(Debug, Clone)]
    struct Foo;

    let mut lib = Library::default();
    lib.define_clone_ty::<Val<Arc<Foo>>>("Foo", "");

    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!(
        r"
        fn f(foo: Foo, idx: i32) {
            if idx > 0 { f(foo, idx - 1); }
        }

        fn main(foo: Foo) -> Verdict[i32, i32] {
            f(foo, 1);
            reject 3
        }
    "
    );

    let mut p = compile_with_runtime(s, rt);
    let f = p
        .func::<fn(Val<Arc<Foo>>) -> Verdict<i32, i32>>("main")
        .unwrap();

    let v = Arc::new(Foo);
    assert_eq!(f.call(Val(v.clone())), Verdict::Reject(3));
    assert_eq!(Arc::strong_count(&v), 1);
}

#[test]
fn str_equals() {
    let s = src!(
        r#"
        fn is_slash(s: String) -> bool {
            s == "/"
        }
        "#
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn(Arc<str>) -> bool>("is_slash").unwrap();

    assert!(func.call("/".into()));
    assert!(!func.call("foo".into()));
}

#[test]
fn str_not_equals() {
    let s = src!(
        r#"
        fn is_not_slash(s: String) -> bool {
            s != "/"
        }
        "#
    );

    let mut compiled = compile(s);
    let func = compiled
        .func::<fn(Arc<str>) -> bool>("is_not_slash")
        .unwrap();

    assert!(func.call("foo".into()));
    assert!(!func.call("/".into()));
}

#[test]
fn assignment() {
    let s = src!(
        "
            fn foo() -> i32 {
                let x = 4;
                x = x + 3;
                x
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 7);
}

#[test]
fn assignment_record_field() {
    let s = src!(
        "
            fn foo() -> i32 {
                let x = { bar: 4 };
                x.bar = x.bar + 3;
                x.bar
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 7);
}

#[test]
fn assignment_record() {
    let s = src!(
        "
            fn foo() -> i32 {
                let x = { bar: 4 };
                x = { bar: x.bar + 3 };
                x.bar
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 7);
}

#[test]
fn assignment_record_is_by_value() {
    let s = src!(
        "
            fn foo() -> i32 {
                let x = { bar: 4 };
                let y = { bar: 5 };
                x = y;
                x.bar = 6;
                y.bar
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 5);
}

#[test]
fn assignment_nested_record_1() {
    let s = src!(
        "
            fn foo() -> i32 {
                let x = { bar: { baz: 1 } };
                x.bar.baz = 6;
                x.bar.baz
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 6);
}

#[test]
fn assignment_nested_record_2() {
    let s = src!(
        "
            fn foo() -> i32 {
                let x = { bar: { baz: 1 } };
                x.bar = { baz: 6 };
                x.bar.baz
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 6);
}

#[test]
fn assignment_string() {
    let s = src!(
        "
            fn foo() -> String {
                let x = \"foo\";
                x = x + x;
                x = x + x;
                x
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> Arc<str>>("foo").unwrap();

    assert_eq!(func.call(), "foofoofoofoo".into());
}

#[test]
fn let_declaration_is_by_value() {
    let s = src!(
        "
            fn foo() -> i32 {
                let y = { bar: 5 };
                let x = y;
                x.bar = 6;
                y.bar
            }
        "
    );

    let mut compiled = compile(s);
    let func = compiled.func::<fn() -> i32>("foo").unwrap();

    assert_eq!(func.call(), 5);
}

#[test]
fn sigill() {
    struct Arcane(Arc<()>);

    impl Clone for Arcane {
        fn clone(&self) -> Self {
            assert!(Arc::strong_count(&self.0) > 0);
            Arcane(self.0.clone())
        }
    }

    impl Drop for Arcane {
        fn drop(&mut self) {
            assert!(Arc::strong_count(&self.0) > 0);
        }
    }

    let rt = Runtime::from_lib({
        let mut impl_block = roto::Impl::new::<Val<Arcane>>();
        impl_block.define_func("get", "", ["self"], |this: Val<Arcane>| -> u64 {
            Arc::strong_count(&this.0.0) as u64
        });

        let mut lib = roto::Library::default();
        lib.define_clone_ty::<Val<Arcane>>("Arcane", "");
        lib.add(impl_block.into());
        lib.define_func("make_arcane", "", [], || -> Val<Arcane> {
            Val(Arcane(Arc::new(())))
        });
        lib
    })
    .unwrap();

    let s = src!(
        "
        fn bar(a: Arcane) -> u64 {
            a.get()
        }


        fn foo() -> Arcane {
            let a = make_arcane();
            bar(a);
            bar(a);
            a
        }
    "
    );

    let mut compiled = compile_with_runtime(s, rt);
    let func = compiled.func::<fn() -> Val<Arcane>>("foo").unwrap();
    let Val(a) = func.call();
    assert_eq!(Arc::strong_count(&a.0), 1);
}

/// The normal Rust string type will fail more often than `Arc<str>`
/// if we treat it wrong, so this test stress-tests Roto a bit with that
/// type.
#[test]
fn rust_string_string() {
    let mut impl_block = roto::Impl::new::<Val<String>>();
    impl_block.define_func("new", "", ["s"], |s: Arc<str>| -> Val<String> {
        Val(s.as_ref().into())
    });

    let rt = Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_clone_ty::<Val<String>>("RustString", "");
        lib.add(impl_block.into());
        lib
    })
    .unwrap();

    let s = src!(
        "
            fn foo() -> RustString {
                let s = RustString.new(\"hello\");
                s = s;
                s
            }

            fn bar() -> RustString {
                let s = RustString.new(\"hello\");
                let a = { str: s };
                a.str = a.str;
                a.str
            }
        "
    );

    let mut compiled = compile_with_runtime(s, rt);
    let func = compiled.func::<fn() -> Val<String>>("foo").unwrap();
    assert_eq!(func.call(), Val("hello".into()));

    let func = compiled.func::<fn() -> Val<String>>("bar").unwrap();
    assert_eq!(func.call(), Val("hello".into()));
}

#[test]
fn return_verdict_from_runtime_function() {
    let mut lib = Library::default();
    lib.define_func("foo", "", [], || -> Verdict<(), ()> { Verdict::Accept(()) });

    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!("filtermap bar() { foo() }");

    let mut compiled = compile_with_runtime(s, rt);
    let func = compiled.func::<fn() -> Verdict<(), ()>>("bar").unwrap();

    assert_eq!(Verdict::Accept(()), func.call());
}

#[test]
fn string_global() {
    let s = src!("fn use_foo() -> String { FOO }");

    let mut lib = roto::Library::default();
    lib.define_const::<Arc<str>>("FOO", "", "BAR".into());

    let rt = Runtime::from_lib(lib).unwrap();

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn() -> Arc<str>>("use_foo").unwrap();

    assert_eq!(f.call(), "BAR".into());
}

// Originally from issue #234
#[test]
fn layered_option_matching_none() {
    let s = src!(
        "
        fn reproducer() -> String? {
          let s = match None {
            Some(v) -> v,
            None -> { return None; },
          };
          Some(s)
        }
        "
    );
    let mut p = compile(s);
    let f = p.func::<fn() -> Option<Arc<str>>>("reproducer").unwrap();
    assert_eq!(f.call(), None);
}

#[test]
fn strings_from_if() {
    let s = src!(
        r#"
        fn foo(x: bool) -> String {
            if x { "true" } else { "false" }
        }
        "#
    );

    let mut p = compile(s);
    let f = p.func::<fn(bool) -> Arc<str>>("foo").unwrap();

    assert_eq!(f.call(false), "false".into());
    assert_eq!(f.call(true), "true".into());
}

#[test]
fn bool_is_true() {
    let s = src!(
        r"
        fn foo(x: bool) -> bool {
            x == true
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(bool) -> bool>("foo").unwrap();

    assert!(!f.call(false));
    assert!(f.call(true));
}

#[test]
fn bool_is_not_true() {
    let s = src!(
        r"
        fn foo(x: bool) -> bool {
            x != true
        }
        "
    );

    let mut p = compile(s);
    let f = p.func::<fn(bool) -> bool>("foo").unwrap();

    assert!(f.call(false));
    assert!(!f.call(true));
}

#[test]
fn register_on_optstr() {
    let mut impl_block = roto::Impl::new::<Val<Option<Arc<str>>>>();
    impl_block.define_func(
        "unwrap_or_empty",
        "",
        ["self"],
        |this: Val<Option<Arc<str>>>| -> Arc<str> { this.0.unwrap_or_default() },
    );

    let rt = Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_clone_ty::<Val<Option<Arc<str>>>>("OptStr", "");
        lib.add(impl_block.into());
        lib
    })
    .unwrap();

    let s = src!("fn foo(x: OptStr) -> String { x.unwrap_or_empty() }");

    let mut p = compile_with_runtime(s, rt);
    let f = p
        .func::<fn(Val<Option<Arc<str>>>) -> Arc<str>>("foo")
        .unwrap();

    assert_eq!(f.call(Val(None)), "".into());
    assert_eq!(f.call(Val(Some("hello".into()))), "hello".into());
}

#[test]
fn register_closure() {
    let some_number = 10;

    let mut lib = Library::default();
    lib.define_func("get", "", [], move || some_number);

    let rt = Runtime::from_lib(lib).unwrap();

    let s = src!("fn foo() -> i32 { get() }");

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn() -> i32>("foo").unwrap();
    assert_eq!(f.call(), 10);
}

#[test]
fn increment_via_closure() {
    static COUNTER: AtomicI32 = AtomicI32::new(0);

    let mut lib = Library::default();
    lib.define_func("inc", "", [], || COUNTER.fetch_add(1, Ordering::Relaxed));

    let runtime = Runtime::from_lib(lib).unwrap();

    let s = src!("fn foo() { inc(); inc(); } ");

    let mut p = compile_with_runtime(s, runtime);
    let f = p.func::<fn()>("foo").unwrap();

    f.call();
    assert_eq!(COUNTER.load(Ordering::Relaxed), 2);
}

#[test]
fn f32_issue() {
    let s = src!(
        r"
       fn foo() {
           let x = 5.0;
           let y: f32 = 2.0;
           let z: f32 = x * y;
       }
    "
    );

    let mut lib = Library::default();
    lib.define_func("rand", "", [], || -> f32 { 10.0 });

    let rt = Runtime::from_lib(lib).unwrap();

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn()>("foo").unwrap();

    f.call();
}

#[test]
fn register_module() {
    let s = src!(
        "
       import math.{PI, sin, cos};

       fn foo() -> f32 {
           cos(PI) + sin(PI)
       }
    "
    );

    let mut module = roto::Module::new("math", "").unwrap();
    module.define_const::<f32>("PI", "", std::f32::consts::PI);
    module.define_func(
        "sin",
        "The sine of the radian argument x.",
        ["x"],
        |x: f32| -> f32 { x.sin() },
    );
    module.define_func(
        "cos",
        "The cosine of the radian argument x.",
        ["x"],
        |x: f32| -> f32 { x.cos() },
    );

    let rt = Runtime::from_lib(Library::default().with(module)).unwrap();

    let mut p = compile_with_runtime(s, rt);
    let f = p.func::<fn() -> f32>("foo").unwrap();

    let res = f.call();
    assert!(-1.1 < res && res < -0.9);
}

#[test]
fn assignment_with_question_mark() {
    let s = src!(
        r#"
        fn foo() -> ()? {
            let x: String? = None;
            let y = "hello";
            y = x?;
            Some(())
        }
    "#
    );
    let mut pkg = compile(s);
    let f = pkg.func::<fn() -> Option<()>>("foo").unwrap();
    f.call();
}

#[test]
fn define_variant_type() {
    let s = src!(
        "
       variant Foo {
           Bar,
           Baz,
       }

       fn make_foo(x: bool) -> Foo {
           if x {
               Foo.Bar
           } else {
               Foo.Baz
           }
       }

       fn match_on_foo(x: bool) -> i32 {
           match make_foo(x) {
               Bar -> 10,
               Baz -> 20,
           }
       }
    "
    );

    let mut pkg = compile(s);
    let f = pkg.func::<fn(bool) -> i32>("match_on_foo").unwrap();

    assert_eq!(f.call(true), 10);
    assert_eq!(f.call(false), 20);
}

#[test]
fn haskeller_wants_to_feel_at_home() {
    let s = src!(
        "
       variant Maybe {
           Just(i32),
           Nothing,
       }

       import Maybe.{Just, Nothing};

       fn from_option(x: i32?) -> Maybe {
           match x {
               None -> Nothing,
               Some(x) -> Just(x)
           }
       }

       fn to_option(x: Maybe) -> i32? {
           match x {
               Nothing -> None,
               Just(x) -> Some(x),
           }
       }

       fn useless(x: i32?) -> i32? {
           to_option(from_option(x))
       }
    "
    );

    let mut pkg = compile(s);
    let f = pkg
        .func::<fn(Option<i32>) -> Option<i32>>("useless")
        .unwrap();

    assert_eq!(f.call(Some(5)), Some(5));
    assert_eq!(f.call(None), None);
}

#[test]
fn generic_haskeller_wants_to_feel_at_home() {
    let s = src!(
        "
       variant Maybe[T] {
           Just(T),
           Nothing,
       }

       import Maybe.{Just, Nothing};

       fn from_option(x: i32?) -> Maybe[i32] {
           match x {
               None -> Nothing,
               Some(x) -> Just(x)
           }
       }

       fn to_option(x: Maybe[i32]) -> i32? {
           match x {
               Nothing -> None,
               Just(x) -> Some(x),
           }
       }

       fn useless(x: i32?) -> i32? {
           to_option(from_option(x))
       }
    "
    );

    let mut pkg = compile(s);
    let f = pkg
        .func::<fn(Option<i32>) -> Option<i32>>("useless")
        .unwrap();

    assert_eq!(f.call(Some(5)), Some(5));
    assert_eq!(f.call(None), None);
}

#[test]
fn match_on_empty_variant() {
    let s = src!(
        "
        variant Foo {}

        fn foo(x: Foo) {
            match x {}
        }
    "
    );

    let _pkg = compile(s);
}

#[test]
fn match_on_empty_variant_2() {
    let s = src!(
        "
        variant Foo {}

        fn foo() {
            let x: Foo = return;
            match x {}
        }
    "
    );

    let mut pkg = compile(s);
    let f = pkg.func::<fn() -> ()>("foo").unwrap();
    f.call();
}

#[test]
fn match_on_uninhabited_variant() {
    let s = src!(
        "
        variant Foo { Baz(!) }

        fn foo(x: Foo) {
            match x {
                Baz(x) -> {}
            }
        }
    "
    );

    let _pkg = compile(s);
}

#[test]
fn lets_make_a_result() {
    let s = src!(
        "
        variant Result[T, E] {
            Ok(T),
            Err(E),
        }

        import Result.{Ok, Err};

        fn from(x: Verdict[i32, u32]) -> Result[i32, u32] {
           match x {
               Reject(x) -> Err(x),
               Accept(x) -> Ok(x)
           }
        }

        fn to(x: Result[i32, u32]) -> Verdict[i32, u32] {
           match x {
               Ok(x) -> Verdict.Accept(x),
               Err(x) -> Verdict.Reject(x),
           }
        }

        fn useless(x: Verdict[i32, u32]) -> Verdict[i32, u32] {
           to(from(x))
        }
    "
    );

    let mut pkg = compile(s);
    let f = pkg
        .func::<fn(Verdict<i32, u32>) -> Verdict<i32, u32>>("useless")
        .unwrap();
    assert_eq!(f.call(Verdict::Accept(2)), Verdict::Accept(2));
    assert_eq!(f.call(Verdict::Reject(4)), Verdict::Reject(4));
}

#[test]
fn variant_with_unused_type_param() {
    let s = src!(
        r"
          variant Foo[T] { Bar }

          fn foo() {
              let x = Foo.Bar;
          }
        "
    );
    let mut pkg = compile(s);
    let f = pkg.func::<fn()>("foo").unwrap();
    f.call();
}

#[test]
fn variant_with_never_type_param() {
    let s = src!(
        r"
          variant Foo[T] { Bar }

          fn foo() {
              let x: Foo[!] = Foo.Bar;
          }
        "
    );
    let mut pkg = compile(s);
    pkg.func::<fn()>("foo").unwrap().call();
}

#[test]
fn generic_record() {
    let s = src!(
        "
        record Foo[T] {
            x: T,
        }

        fn wrap(x: i32) -> Foo[i32] {
            Foo { x: x }
        }

        fn unwrap(x: Foo[i32]) -> i32 {
            x.x
        }

        fn bar(x: i32) -> i32 {
            let y = wrap(x);
            unwrap(y)
        }
    "
    );
    let mut pkg = compile(s);
    let f = pkg.func::<fn(i32) -> i32>("bar").unwrap();
    assert_eq!(f.call(23), 23);
}

#[test]
fn generic_record_contains_option() {
    let s = src!(
        "
        record Foo[T] {
            x: T?,
        }

        fn wrap(x: i32?) -> Foo[i32] {
            Foo { x: x }
        }

        fn unwrap(x: Foo[i32]) -> i32? {
            x.x
        }

        fn bar(x: i32?) -> i32? {
            let y = wrap(x);
            unwrap(y)
        }
    "
    );
    let mut pkg = compile(s);
    let f = pkg.func::<fn(Option<i32>) -> Option<i32>>("bar").unwrap();
    assert_eq!(f.call(None), None);
    assert_eq!(f.call(Some(20)), Some(20));
}

#[test]
fn generic_record_inferred() {
    let s = src!(
        "
        record Foo[T] {
            x: T,
        }

        fn wrap(x: i32) -> Foo[i32] {
            { x: x } # this is now inferred to be Foo[i32]
        }

        fn unwrap(x: Foo[i32]) -> i32 {
            x.x
        }

        fn bar(x: i32) -> i32 {
            let y = wrap(x);
            unwrap(y)
        }
    "
    );

    let mut pkg = compile(s);
    let f = pkg.func::<fn(i32) -> i32>("bar").unwrap();
    assert_eq!(f.call(23), 23);
}

#[test]
fn generics_all_the_way_down() {
    let s = src!(
        "
        record Foo[T] {
            bar: Bar[T]?
        }

        record Bar[T] {
            inner: T
        }

        fn main(x: i32) -> i32 {
            let foo = Foo {
                bar: Some(Bar {
                    inner: x
                }),
            };
            match foo.bar {
                Some(bar) -> bar.inner,
                None -> 0,
            }
        }
    "
    );

    let mut pkg = compile(s);
    let f = pkg.func::<fn(i32) -> i32>("main").unwrap();
    assert_eq!(f.call(23), 23);
}
