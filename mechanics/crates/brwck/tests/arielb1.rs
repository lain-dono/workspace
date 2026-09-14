use brwck::repr::*;

#[test]
fn loop_carry_drop() {
    let mut func = Func::default();

    let p_region = RegionName("p");
    let foo_region = RegionName("foo");
    let bar_region = RegionName("bar");

    let foo = func.variable("foo", Ty::Unit);
    let bar = func.variable("bar", Ty::Unit);
    let p = func.variable("p", Ty::ptr_ref(p_region, Ty::Unit));

    let start_block = func.block("START", |build| {
        build.borrow_ref(p, foo_region, foo); // borrows foo
        build.goto(["B"]);
    });

    let b_block = func.block("B", |build| {
        build.goto(["C", "D", "EXIT"]);
    });

    // Here, we could mutate `foo`
    // and `bar` before `p = &`,
    // and we can mutate only `foo` afterwards.
    let c_block = func.block("C", |build| {
        build.noop();
        build.borrow_ref(p, bar_region, bar); // borrows bar
        build.noop();
        build.goto(["D"]);
    });

    let d_block = func.block("D", |build| {
        build.using(p);
        build.goto(["E"]);
    });

    // Here, the resource bar would get
    // dropped. Therefore, we must ensure
    // that E/0 is part of `p@C/2.0`.
    let e_block = func.block("E", |build| {
        build.noop();
        build.goto(["B"]);
    });

    let exit_block = func.block("EXIT", |_| {});

    // cannot drop `bar` in block E:
    func.assert_region_in(bar_region, Point::code(e_block, 0));

    // can mutate `foo` in block C:
    func.assert_region_not_in(foo_region, Point::code(c_block, 0));
    func.assert_region_not_in(foo_region, Point::code(c_block, 2));

    func.assert_var_not_live(p, start_block);
    func.assert_var_live(p, b_block);
    func.assert_var_not_live(p, c_block);
    func.assert_var_live(p, d_block);
    func.assert_var_live(p, e_block);
    func.assert_var_not_live(p, exit_block);

    func.test();
}
