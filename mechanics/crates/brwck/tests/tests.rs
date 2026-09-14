use brwck::repr::*;

// Test that a kill of `b` doesn't (necessarily) consitute a kill of `'x`,
// which is a region used in two places. But if no variables are live that
// use `'x`, then it is dead.
#[test]
fn liveness() {
    let mut func = Func::default();

    let x_region = RegionName("x");

    let a = func.variable("a", Ty::ptr_ref(x_region, Ty::Unit));
    let b = func.variable("b", Ty::ptr_ref(x_region, Ty::Unit));

    let start = func.block("START", |block| {
        block.init(a, None);
        block.goto(["USE"]);
    });

    let usage = func.block("USE", |block| {
        block.init(b, None);
        block.using(a);
        block.using(b);
        block.goto(["END"]);
    });

    let _end = func.block("END", |_| {});

    func.assert_region_live(x_region, usage);
    func.assert_region_not_live(x_region, start);

    func.test();
}

// Test that we handle loans that go across basic blocks correctly.
#[test]
fn multiblock_loan() {
    let mut func = Func::default();

    let foo_struct = func.decl_struct("Foo", vec![], vec![]);

    let foo_region = RegionName("foo");

    let foo = func.variable("foo", Ty::struct0(foo_struct));
    let bar = func.variable("bar", Ty::struct0(foo_struct));

    let a_region = RegionName("a");
    let b_region = RegionName("b");
    let a_inner_region = RegionName("aa");
    let b_inner_region = RegionName("bb");

    let a_inner = func.variable("aa", Ty::ptr_mut(a_inner_region, foo_struct));
    let a = func.variable(
        "a",
        Ty::ptr_mut(a_region, Ty::ptr_mut(a_inner_region, foo_struct)),
    );
    let b = func.variable(
        "b",
        Ty::ptr_mut(b_region, Ty::ptr_mut(b_inner_region, foo_struct)),
    );

    func.block("START", |block| {
        block.init(bar, None);
        block.init(foo, None);
        block.borrow_mut(a_inner, a_inner_region, bar);
        block.borrow_mut(a, a_region, a_inner);
        block.goto(["X"]);
    });

    let x_block = func.block("X", |block| {
        block.borrow_mut(b, b_region, Path::star_var(a));
        block.goto(["END"]);
    });

    let end_block = func.block("END", |block| {
        block.borrow_mut(Path::star_var(a), foo_region, foo);
        block.expect_error("`*a` is borrowed");
        block.using(b);
    });

    func.assert_region_in(b_region, Point::code(x_block, 1));
    func.assert_var_live(b, end_block);

    func.test();

    panic!();
}
