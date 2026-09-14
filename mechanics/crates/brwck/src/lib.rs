#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::similar_names)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::wildcard_imports)]

pub mod borrowck;
pub mod env;
pub mod errors;
pub mod graph;
pub mod infer;
pub mod liveness;
pub mod loans_in_scope;
pub mod region;
pub mod regionck;
pub mod repr;

pub mod graph_impl;

use self::repr::*;

impl Func {
    pub fn test(self) {
        use self::{
            env::Environment,
            graph::{FuncGraph, with_graph},
            regionck::region_check,
        };

        let graph = FuncGraph::new(self);
        with_graph(&graph, || {
            let env = Environment::new(&graph);
            env.dump_dominators();
            println!("Testing...");
            region_check(&env).unwrap();
        });
    }

    pub fn decl_struct(
        &mut self,
        name: &'static str,
        params: Vec<StructParam>,
        fields: Vec<StructField>,
    ) -> StructName {
        let name = StructName(name);
        self.structs.push(StructDecl {
            name,
            params,
            fields,
        });
        name
    }

    pub fn variable(&mut self, var: &'static str, ty: impl Into<Ty>) -> Variable {
        let (var, ty) = (Variable(var), ty.into());
        self.decls.push(VariableDecl { var, ty });
        var
    }

    pub fn block(
        &mut self,
        name: &'static str,
        block: impl FnOnce(&mut BlockBuilder),
    ) -> BlockName {
        let mut builder = BlockBuilder::default();

        block(&mut builder);

        let name = BlockName(name);
        let block = BlockDecl {
            name,
            actions: builder.actions,
            successors: builder.successors,
        };

        self.data.insert(block.name, block);

        name
    }
}

impl Func {
    pub fn assert_eq(&mut self, name: RegionName, points: impl IntoIterator<Item = Point>) {
        let points = points.into_iter().collect();
        self.assertions
            .push(Assertion::Eq(name, RegionLiteral { points }));
    }

    pub fn assert_region_in(&mut self, name: RegionName, point: Point) {
        self.assertions.push(Assertion::In(name, point));
    }

    pub fn assert_region_not_in(&mut self, name: RegionName, point: Point) {
        self.assertions.push(Assertion::NotIn(name, point));
    }

    pub fn assert_var_live(&mut self, var: Variable, block: BlockName) {
        self.assertions.push(Assertion::Live(var, block));
    }

    pub fn assert_var_not_live(&mut self, var: Variable, block: BlockName) {
        self.assertions.push(Assertion::NotLive(var, block));
    }

    pub fn assert_region_live(&mut self, region: RegionName, block: BlockName) {
        self.assertions.push(Assertion::RegionLive(region, block));
    }

    pub fn assert_region_not_live(&mut self, region: RegionName, block: BlockName) {
        self.assertions
            .push(Assertion::RegionNotLive(region, block));
    }
}

#[derive(Default)]
pub struct BlockBuilder {
    actions: Vec<Action>,
    successors: Vec<BlockName>,
}

impl BlockBuilder {
    pub fn noop(&mut self) {
        self.actions.push(Action {
            kind: ActionKind::Noop,
            should_have_error: None,
        });
    }

    pub fn init(&mut self, p: impl Into<Path>, v: impl IntoIterator<Item = Path>) {
        self.actions.push(Action {
            kind: ActionKind::Init(Box::new(p.into()), v.into_iter().map(Box::new).collect()),
            should_have_error: None,
        });
    }

    pub fn using(&mut self, p: impl Into<Path>) {
        self.actions.push(Action {
            kind: ActionKind::Use(Box::new(p.into())),
            should_have_error: None,
        });
    }

    pub fn dropping(&mut self, p: impl Into<Path>) {
        self.actions.push(Action {
            kind: ActionKind::Drop(Box::new(p.into())),
            should_have_error: None,
        });
    }

    pub fn borrow_ref(&mut self, lhs: impl Into<Path>, region: RegionName, rhs: impl Into<Path>) {
        let [lhs, rhs] = [lhs.into(), rhs.into()].map(Box::new);
        self.actions.push(Action {
            kind: ActionKind::Borrow(lhs, region, BorrowKind::Ref, rhs),
            should_have_error: None,
        });
    }
    pub fn borrow_mut(&mut self, lhs: impl Into<Path>, region: RegionName, rhs: impl Into<Path>) {
        let [lhs, rhs] = [lhs.into(), rhs.into()].map(Box::new);
        self.actions.push(Action {
            kind: ActionKind::Borrow(lhs, region, BorrowKind::Mut, rhs),
            should_have_error: None,
        });
    }

    pub fn assign(&mut self, a: impl Into<Path>, b: impl Into<Path>) {
        self.actions.push(Action {
            kind: ActionKind::Assign(Box::new(a.into()), Box::new(b.into())),
            should_have_error: None,
        });
    }

    pub fn constraint(&mut self, constraint: Constraint) {
        self.actions.push(Action {
            kind: ActionKind::Constraint(Box::new(constraint)),
            should_have_error: None,
        });
    }

    pub fn constraint_outlives(&mut self, sup: RegionName, sub: RegionName) {
        self.constraint(Constraint::Outlives(OutlivesConstraint { sup, sub }));
    }

    pub fn expect_error(&mut self, expect: impl Into<String>) {
        let expect = expect.into();
        self.actions.last_mut().unwrap().should_have_error = Some(ExpectedError { string: expect });
    }

    pub fn goto(&mut self, blocks: impl AsRef<[&'static str]>) {
        for block in blocks.as_ref() {
            self.successors.push(BlockName(block));
        }
    }
}
