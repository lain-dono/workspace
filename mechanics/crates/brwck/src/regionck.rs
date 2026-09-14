use crate::borrowck;
use crate::env::{Environment, Point};
use crate::errors::ErrorReporting;
use crate::infer::{InferenceContext, RegionVariable};
use crate::liveness::Liveness;
use crate::loans_in_scope::LoansInScope;
use crate::region::Region;
use crate::repr::{self, RegionDecl, RegionName, Variance};
use std::collections::HashMap;
use std::error::Error;

pub fn region_check(env: &Environment) -> Result<(), Box<dyn Error>> {
    let mut ck = RegionCheck {
        env,
        infer: InferenceContext::default(),
        region_map: HashMap::new(),
    };
    ck.check()
}

pub struct RegionCheck<'env> {
    env: &'env Environment<'env>,
    infer: InferenceContext,
    region_map: HashMap<repr::RegionName, RegionVariable>,
}

impl<'env> RegionCheck<'env> {
    pub fn env(&self) -> &'env Environment<'env> {
        self.env
    }

    pub fn region(&self, name: RegionName) -> &Region {
        let Some(&var) = self.region_map.get(&name) else {
            panic!("no region variable ever created with name `{name:?}`")
        };
        self.infer.region(var)
    }

    fn check(&mut self) -> Result<(), Box<dyn Error>> {
        let mut errors = ErrorReporting::default();

        // Register expected errors.
        for &block in &self.env.reverse_post_order {
            let actions = self.env.graph.block_data(block).actions();
            for (index, action) in actions.iter().enumerate() {
                let point = Point {
                    block,
                    action: index,
                };
                if let Some(ref expected) = action.should_have_error {
                    errors.expect_error(point, &expected.string);
                }
            }
        }

        // Compute liveness.
        let liveness = Liveness::new(self.env);

        // Add inference constraints.
        self.populate_inference(&liveness);

        // Solve inference constraints, reporting any errors.
        for error in self.infer.solve(self.env) {
            errors.report_error(
                error.constraint_point,
                format!("capped variable `{}` exceeded its limits", error.name),
            );
        }

        // Compute loans in scope at each point.
        let loans_in_scope = &LoansInScope::new(self);

        // Run the borrow check, reporting any errors.
        borrowck::borrow_check(self.env, loans_in_scope, &mut errors);

        // Check that all assertions are obeyed.
        self.check_assertions(&liveness)?;

        // Check that we found the errors we expect to.
        errors.reconcile_errors()?;

        Ok(())
    }

    fn check_assertions(&self, liveness: &Liveness) -> Result<(), String> {
        let mut errors = 0;

        for assertion in self.env.graph.assertions() {
            match assertion {
                repr::Assertion::Eq(region_name, region_literal) => {
                    let region_var = self.region_map[region_name];
                    let region_value = self.to_region(region_literal);
                    if *self.infer.region(region_var) != region_value {
                        errors += 1;
                        println!("error: region variable `{region_name:?}` has wrong value");
                        println!("  expected: {region_value:?}");
                        println!("  found   : {:?}", self.infer.region(region_var));
                    }
                }

                repr::Assertion::In(region_name, point) => {
                    let region_var = self.region_map[region_name];
                    let point = self.to_point(point);
                    if !self.infer.region(region_var).may_contain(point) {
                        errors += 1;
                        println!(
                            "error: region variable `{region_name:?}` does not contain `{point:?}`"
                        );
                        println!("  found   : {:?}", self.infer.region(region_var));
                    }
                }

                repr::Assertion::NotIn(region_name, point) => {
                    let region_var = self.region_map[region_name];
                    let point = self.to_point(point);
                    if self.infer.region(region_var).may_contain(point) {
                        errors += 1;
                        println!("error: region variable `{region_name:?}` contains `{point:?}`");
                        println!("  found   : {:?}", self.infer.region(region_var));
                    }
                }

                &repr::Assertion::Live(var, block_name) => {
                    let block = self.env.graph.block(block_name);
                    if !liveness.var_live_on_entry(var, block) {
                        errors += 1;
                        println!("error: variable `{var:?}` not live on entry to `{block_name:?}`");
                    }
                }

                &repr::Assertion::NotLive(var, block_name) => {
                    let block = self.env.graph.block(block_name);
                    if liveness.var_live_on_entry(var, block) {
                        errors += 1;
                        println!("error: variable `{var:?}` live on entry to `{block_name:?}`");
                    }
                }

                &repr::Assertion::RegionLive(region_name, block_name) => {
                    let block = self.env.graph.block(block_name);
                    if !liveness.region_live_on_entry(region_name, block) {
                        errors += 1;
                        println!(
                            "error: region `{region_name:?}` not live on entry to `{block_name:?}`"
                        );
                    }
                }

                &repr::Assertion::RegionNotLive(region_name, block_name) => {
                    let block = self.env.graph.block(block_name);
                    if liveness.region_live_on_entry(region_name, block) {
                        errors += 1;
                        println!(
                            "error: region `{region_name:?}` live on entry to `{block_name:?}`"
                        );
                    }
                }
            }
        }

        if errors > 0 {
            Err(format!("{errors} errors found"))
        } else {
            Ok(())
        }
    }

    fn populate_outlives(
        &mut self,
        rv: RegionVariable,
        visited: &mut Vec<RegionName>, // memoization
        outlives: &Vec<RegionName>,
    ) {
        for &region in outlives {
            // avoid recomputation
            if visited.contains(&region) {
                continue;
            }

            let block = self.env.graph.skolemized_end(region);
            self.infer.add_live_point(rv, Point { block, action: 0 });

            let outlives = {
                let mut possible_matches = self
                    .env
                    .graph
                    .free_regions()
                    .iter()
                    .filter(|rd| region == rd.name);
                match possible_matches.next() {
                    Some(region_decl) => &region_decl.outlives,
                    None => continue,
                }
            };

            visited.push(region);
            self.populate_outlives(rv, visited, outlives);
        }
    }

    fn populate_inference(&mut self, liveness: &Liveness) {
        // This is sort of a hack, but... for each "free region" `r`,
        // we will wind up with a region variable. We want that region
        // variable to be inferred to precisely the set: `{G, ...,
        // End(r)}`, where `G` is all the points in the control-flow
        // graph, and `End(r)` is the end-point of `r`. We also want
        // to include the endpoints of any free-regions that `r`
        // outlives. We're not enforcing (in inference) that `r` doesn't
        // get inferred to some *larger* region (that would be a kind of
        // constraint we would need to add, and inference right now
        // doesn't permit such constraints -- you could also view it
        // an assertion that we add to the tests).
        for region_decl in self.env.graph.free_regions() {
            let &RegionDecl {
                name: region,
                ref outlives,
            } = region_decl;

            let rv = self.region_variable(region);
            for &block in &self.env.reverse_post_order {
                let end_point = self.env.end_point(block);
                for action in 0..end_point.action {
                    self.infer.add_live_point(rv, Point { block, action });
                }
                self.infer.add_live_point(rv, end_point);
            }

            let block = self.env.graph.skolemized_end(region);
            self.infer.add_live_point(rv, Point { block, action: 0 });
            self.populate_outlives(rv, &mut vec![region], outlives);
            self.infer.cap_var(rv);
            println!("Region for {region:?}:\n{:#?}\n", self.infer.region(rv));
        }

        liveness.walk(|point, action, live_on_entry| {
            // To start, find every variable `x` that is live. All regions
            // in the type of `x` must include `point`.
            for region_name in liveness.live_regions(live_on_entry) {
                let rv = self.region_variable(region_name);
                self.infer.add_live_point(rv, point);
            }

            let Some(action) = action else {
                return;
            };

            // Next, walk the actions and establish any additional constraints
            // that may arise from subtyping.
            let successor_point = Point {
                block: point.block,
                action: point.action + 1,
            };
            match &action.kind {
                // `p = &'x` -- first, `'x` must include this point @ P,
                // and second `&'x <: typeof(p) @ succ(P)`
                &repr::ActionKind::Borrow(ref dst_path, region_name, borrow_kind, ref src_path) => {
                    println!("borrow  {dst_path} {region_name} {src_path}");

                    let dst_ty = self.env.path_ty(dst_path);
                    let src_ty = self.env.path_ty(src_path);

                    println!("\tdst: {dst_ty}");
                    println!("\tsrc: {src_ty}");

                    let region = repr::Region::Free(region_name);
                    let ref_ty = match borrow_kind {
                        repr::BorrowKind::Ref => repr::Ty::Ref(region, Box::new(src_ty)),
                        repr::BorrowKind::Mut => repr::Ty::Mut(region, Box::new(src_ty)),
                    };

                    println!("\tref: {ref_ty}");

                    self.relate_tys(successor_point, repr::Variance::Contra, &dst_ty, &ref_ty);
                    self.ensure_borrow_source(successor_point, region_name, src_path);
                }

                // a = b
                repr::ActionKind::Assign(lhs, rhs) => {
                    let lhs_ty = self.env.path_ty(lhs);
                    let rhs_ty = self.env.path_ty(rhs);

                    // `b` must be a subtype of `a` to be assignable:
                    self.relate_tys(successor_point, repr::Variance::Co, &rhs_ty, &lhs_ty);
                }

                // 'X: 'Y
                repr::ActionKind::Constraint(c) => match c.as_ref() {
                    repr::Constraint::Outlives(c) => {
                        let sup_v = self.region_variable(c.sup);
                        let sub_v = self.region_variable(c.sub);
                        self.infer.add_outlives(sup_v, sub_v, point);
                    }
                    _ => panic!("unimplemented rich constraint: {c:?}"),
                },

                repr::ActionKind::Init(..)
                | repr::ActionKind::Use(..)
                | repr::ActionKind::Drop(..)
                | repr::ActionKind::StorageDead(..)
                | repr::ActionKind::SkolemizedEnd(_)
                | repr::ActionKind::Noop => {
                    // no add'l constriants needed here; basic liveness
                    // suffices.
                }
            }
        });
    }

    fn region_variable(&mut self, n: repr::RegionName) -> RegionVariable {
        let infer = &mut self.infer;
        let r = *self.region_map.entry(n).or_insert_with(|| infer.add_var(n));
        println!("{n} => {r:?}");
        r
    }

    fn to_point(&self, point: &repr::Point) -> Point {
        let action = point.action;
        let block = match point.block {
            repr::PointName::Code(b) => self.env.graph.block(b),
            repr::PointName::SkolemizedEnd(r) => self.env.graph.skolemized_end(r),
        };
        Point { block, action }
    }

    fn to_region(&self, user_region: &repr::RegionLiteral) -> Region {
        let mut region = Region::default();
        for p in &user_region.points {
            region.add_point(self.to_point(p));
        }
        region
    }

    fn relate_tys(
        &mut self,
        successor_point: Point,
        variance: repr::Variance,
        a: &repr::Ty,
        b: &repr::Ty,
    ) {
        println!("relate_tys({a} {variance:?} {b} @ {successor_point:?})");
        match (a, b) {
            (repr::Ty::Ref(r_a, t_a), repr::Ty::Ref(r_b, t_b)) => {
                self.relate_regions(
                    successor_point,
                    variance.invert(),
                    r_a.assert_free(),
                    r_b.assert_free(),
                );
                let referent_variance = variance.xform(repr::BorrowKind::Ref.variance());
                self.relate_tys(successor_point, referent_variance, t_a, t_b);
            }
            (repr::Ty::Mut(r_a, t_a), repr::Ty::Ref(r_b, t_b)) => {
                self.relate_regions(
                    successor_point,
                    variance.invert(),
                    r_a.assert_free(),
                    r_b.assert_free(),
                );
                let referent_variance = variance.xform(repr::BorrowKind::Mut.variance());
                self.relate_tys(successor_point, referent_variance, t_a, t_b);
            }

            (repr::Ty::Unit, repr::Ty::Unit) => {}
            (repr::Ty::Struct(s_a, ps_a), repr::Ty::Struct(s_b, ps_b)) => {
                assert_eq!(s_a, s_b, "cannot compare `{s_a:?}` and `{s_b:?}`");
                let s_decl = self.env.struct_map[s_a];
                assert_eq!(
                    ps_a.len(),
                    s_decl.params.len(),
                    "wrong number of parameters for `{a}`"
                );
                assert_eq!(
                    ps_b.len(),
                    s_decl.params.len(),
                    "wrong number of parameters for `{b}`"
                );
                for (sp, (p_a, p_b)) in s_decl.params.iter().zip(ps_a.iter().zip(ps_b)) {
                    let v = variance.xform(sp.variance);
                    self.relate_parameters(successor_point, v, p_a, p_b);
                }
            }
            (a, b) => panic!("cannot relate types `{a}` and `{b}` at {successor_point:?}"),
        }
    }

    fn relate_regions(
        &mut self,
        successor_point: Point,
        variance: repr::Variance,
        a: repr::RegionName,
        b: repr::RegionName,
    ) {
        println!("relate_regions({a} {variance:?} {b} @ {successor_point:?})");
        let r_a = self.region_variable(a);
        let r_b = self.region_variable(b);
        match variance {
            // "a Co b" == "a <= b"
            Variance::Co => self.infer.add_outlives(r_b, r_a, successor_point),
            // "a Contra b" == "a >= b"
            Variance::Contra => self.infer.add_outlives(r_a, r_b, successor_point),
            Variance::In => {
                self.infer.add_outlives(r_a, r_b, successor_point);
                self.infer.add_outlives(r_b, r_a, successor_point);
            }
        }
    }

    fn relate_parameters(
        &mut self,
        successor_point: Point,
        variance: repr::Variance,
        a: &repr::TyParam,
        b: &repr::TyParam,
    ) {
        match (a, b) {
            (repr::TyParam::Ty(t_a), repr::TyParam::Ty(t_b)) => {
                self.relate_tys(successor_point, variance, t_a, t_b);
            }
            (repr::TyParam::Region(r_a), repr::TyParam::Region(r_b)) => self.relate_regions(
                successor_point,
                variance,
                r_a.assert_free(),
                r_b.assert_free(),
            ),
            _ => panic!("cannot relate parameters `{a:?}` and `{b:?}`"),
        }
    }

    /// Add any relations between regions that are needed to ensures
    /// that reborrows live long enough. Specifically, if we borrow
    /// something like `*r` for `'a`, where `r: &'b i32`, then `'b:
    /// 'a` is required.
    fn ensure_borrow_source(
        &mut self,
        successor_point: Point,
        borrow_region_name: RegionName,
        source_path: &repr::Path,
    ) {
        println!(
            "ensure_borrow_source({successor_point:?}, {borrow_region_name:?}, {source_path:?})"
        );

        for supporting_path in self.env.supporting_prefixes(source_path) {
            match supporting_path {
                repr::Path::Var(_) => {
                    // No lifetime constraints are needed to ensure the
                    // validity of a variable. That is ensured by borrowck
                    // preventing the storage of variables from being killed
                    // while data owned by that variable is in scope.
                    return;
                }
                repr::Path::Extension(base_path, field_name) => {
                    let ty = self.env.path_ty(base_path);
                    println!("ensure_borrow_source: ty={ty:?}");
                    match ty {
                        repr::Ty::Ref(ref_region, _) | repr::Ty::Mut(ref_region, _) => {
                            assert_eq!(field_name, &repr::FieldName::STAR);
                            let ref_region_name = ref_region.assert_free();
                            let borrow_region_variable = self.region_variable(borrow_region_name);
                            let ref_region_variable = self.region_variable(ref_region_name);
                            self.infer.add_outlives(
                                ref_region_variable,
                                borrow_region_variable,
                                successor_point,
                            );
                        }
                        repr::Ty::Unit | repr::Ty::Struct(..) | repr::Ty::Bound(..) => {}
                    }
                }
            }
        }
    }
}
