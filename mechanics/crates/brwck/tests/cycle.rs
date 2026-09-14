use brwck::repr::*;

// Test that we can construct a cycle, if `Foo` is may-dangle.
#[test]
fn cycle_good() {
    let mut func = Func::default();

    let zero = Region::Bound(0);
    let foo = RegionName("foo");
    let borrow = RegionName("borrow");
    let p0 = RegionName("p0");
    let p1 = RegionName("p1");

    let foo_type = {
        let name = "Foo";

        let ptr = Ty::ptr_ref(zero, Ty::struct1(name, zero));
        let cell = Ty::struct1("Cell", Ty::struct1("Option", ptr));
        let field = StructField::new("c", cell);
        let param = StructParam::region(Variance::In, true);

        func.decl_struct(name, vec![param], vec![field])
    };

    let foo_var = func.variable("foo", Ty::struct1(foo_type, foo));
    let p_var = func.variable("p", Ty::ptr_ref(p0, Ty::struct1(foo_type, p1)));

    let start = func.block("START", |block| {
        block.init(foo_var, None); // foo = Foo { c: Cell::new(None) };

        block.borrow_ref(p_var, borrow, foo_var); // p = &foo;

        block.constraint_outlives(p0, foo);
        block.constraint_outlives(p1, foo); // foo.c.set(Some(p));

        block.using(p_var);
        block.using(foo_var);

        block.goto(["END"]);
    });

    let end_block = func.block("END", |block| block.dropping(foo_var));

    // In particular, at the time when drop `foo`, it is NOT considered borrowed:
    func.assert_region_not_in(borrow, Point::code(end_block, 0));

    func.assert_region_not_in(foo, Point::code(end_block, 0));
    func.assert_region_not_in(p0, Point::code(end_block, 0));
    func.assert_region_not_in(p1, Point::code(end_block, 0));

    func.assert_eq(
        foo,
        [
            Point::code(start, 1),
            Point::code(start, 2),
            Point::code(start, 3),
            Point::code(start, 4),
            Point::code(start, 5),
        ],
    );

    for region in [p0, p1, borrow] {
        let points = [
            Point::code(start, 2),
            Point::code(start, 3),
            Point::code(start, 4),
            Point::code(start, 5),
        ];
        func.assert_eq(region, points);
    }

    func.test();
}

// Test that we cannot construct a cycle, if `Foo` is not may-dangle.
#[test]
fn cycle_bad() {
    let mut func = Func::default();

    let zero = Region::Bound(0);
    let foo = RegionName("foo");
    let borrow = RegionName("borrow");
    let p0 = RegionName("p0");
    let p1 = RegionName("p1");

    // NB: no may_dangle attribute
    let foo_type = {
        let name = "Foo";

        let ptr = Ty::ptr_ref(zero, Ty::struct1(name, zero));
        let cell = Ty::struct1("Cell", Ty::struct1("Option", ptr));
        let field = StructField::new("c", cell);
        let param = StructParam::region(Variance::In, false);

        func.decl_struct(name, vec![param], vec![field])
    };

    let foo_var = func.variable("foo", Ty::struct1(foo_type, foo));
    let p_var = func.variable("p", Ty::ptr_ref(p0, Ty::struct1(foo_type, p1)));

    let _ = func.block("START", |block| {
        block.init(foo_var, None); // foo = Foo { c: Cell::new(None) };

        block.borrow_ref(p_var, borrow, foo_var); // p = &foo;

        block.constraint_outlives(p0, foo);
        block.constraint_outlives(p1, foo); // foo.c.set(Some(p));

        block.using(p_var);
        block.using(foo_var);

        block.goto(["END"]);
    });

    let end = func.block("END", |block| {
        block.dropping(foo_var);
        block.expect_error("`foo` is borrowed");
    });

    // At the time when we drop `foo`, it is considered borrowed:
    func.assert_region_in(borrow, Point::code(end, 0));

    func.test();
}
