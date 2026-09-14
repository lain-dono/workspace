#![allow(unused_imports)]

use roto::{Runtime, Val};
use routecore::bgp::{
    aspath::{AsPath, HopPath},
    communities::{Community, Wellknown},
    types::{LocalPref, OriginType},
};
use std::sync::Arc;

#[test]
#[should_panic]
fn invalid_function_name() {
    let mut lib = roto::Library::default();
    lib.define_func("accept", "", [], || true);
}

#[test]
#[should_panic]
fn invalid_method_name() {
    let mut impl_block = roto::Impl::new::<bool>();
    impl_block.define_func("accept", "", ["_x"], |_: bool| true);

    let mut lib = roto::Library::default();
    lib.add(impl_block.into());
}

#[test]
#[should_panic]
fn invalid_static_method_name() {
    let mut impl_block = roto::Impl::new::<bool>();
    impl_block.define_func("accept", "", [], || true);

    let mut lib = roto::Library::default();
    lib.add(impl_block.into());
}

#[test]
#[should_panic]
fn invalid_type_name() {
    #[derive(Clone, Copy)]
    struct Foo;

    let mut lib = roto::Library::default();
    lib.define_clone_ty::<Val<Foo>>("accept", "");

    let mut lib = roto::Library::default();
    lib.define_copy_ty::<Val<Foo>>("accept", "");
}

#[test]
#[should_panic]
fn invalid_constant_name() {
    let mut lib = roto::Library::default();
    lib.define_const::<u32>("accept", "", 0);
}

#[test]
fn constant_declared_twice() {
    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_const::<u32>("FOO", "", 10);
        lib.define_const::<u32>("FOO", "", 12);
        lib
    })
    .unwrap_err();
}

#[test]
fn function_declared_twice() {
    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_func("foo", "", [], || ());
        lib.define_func("foo", "", [], || false);
        lib
    })
    .unwrap_err();
}

#[test]
fn method_declared_twice() {
    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_func("foo", "", ["_"], |_: bool| ());
        lib.define_func("foo", "", ["_"], |_: bool| false);
        lib
    })
    .unwrap_err();
}

#[test]
fn static_method_declared_twice() {
    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_func("foo", "", [], || ());
        lib.define_func("foo", "", [], || false);
        lib
    })
    .unwrap_err();
}

#[test]
fn method_and_static_method_with_the_same_name() {
    let mut impl_block = roto::Impl::new::<bool>();
    impl_block.define_func("foo", "", ["_"], |_: bool| ());
    impl_block.define_func("foo", "", [], || false);

    Runtime::from_lib(roto::Library::default().with(impl_block)).unwrap_err();
}

#[test]
fn function_and_method_with_the_same_name() {
    let mut impl_block = roto::Impl::new::<bool>();
    impl_block.define_func("foo", "", ["_"], |_: bool| false);

    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_func("foo", "", ["_"], |_: bool| {});
        lib.add(impl_block.into());
        lib
    })
    .unwrap();
}

#[test]
fn function_and_constant_with_the_same_name_1() {
    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_func("foo", "", ["_"], |_: bool| ());
        lib.define_const::<bool>("foo", "", true);
        lib
    })
    .unwrap_err();
}

#[test]
fn function_and_constant_with_the_same_name_2() {
    Runtime::from_lib({
        let mut lib = roto::Library::default();
        lib.define_const::<bool>("foo", "", true);
        lib.define_func("foo", "", ["_x"], |_x: bool| ());
        lib
    })
    .unwrap_err();
}

#[test]
#[should_panic]
fn register_option_arc_str() {
    // Cannot register Option
    let mut lib = roto::Library::default();
    lib.define_clone_ty::<Option<Arc<str>>>("OptStr", "");
}

#[test]
fn register_val_option_arc_str() {
    let mut lib = roto::Library::default();
    lib.define_clone_ty::<Val<Option<Arc<str>>>>("OptStr", "");

    // But with Val it's fine
    Runtime::from_lib(lib).unwrap();
}

// This is a bit of a weird case, it should probably at least warn, but
// at the moment, this is perfectly valid (where unwrap_or_empty is a
// static method and not a method).
#[test]
fn unwrap_or_empty() {
    let mut impl_block = roto::Impl::new::<Val<Option<Arc<str>>>>();
    impl_block.define_func(
        "unwrap_or_empty",
        "",
        ["x"],
        |x: Option<Arc<str>>| -> Arc<str> { x.unwrap_or_default() },
    );

    let mut lib = roto::Library::default();
    lib.define_clone_ty::<Val<Option<Arc<str>>>>("OptStr", "");
    lib.add(impl_block.into());

    Runtime::from_lib(lib).unwrap();
}
