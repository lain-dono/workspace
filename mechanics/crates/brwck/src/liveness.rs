use crate::env::{Environment, Point};
use crate::graph::{BlockIndex, FuncGraph};
use crate::graph_impl::{BitBuf, BitSet, BitSlice, Graph};
use crate::repr;
use std::collections::{BTreeSet, HashMap};
use std::iter::once;

/// Compute the set of live variables at each point.
pub struct Liveness<'env> {
    env: &'env Environment<'env>,
    bits: Vec<BitKind>,
    bits_map: HashMap<BitKind, usize>,
    liveness: BitSet<FuncGraph>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BitKind {
    /// If this bit is set, current value of the variable will be **used** later on.
    VariableUsed(repr::Variable),

    /// If this bit is set, current value of the variable will be **dropped** later on.
    VariableDrop(repr::Variable),

    /// If this bit is set, then the given free region will be
    /// **used**.
    FreeRegion(repr::RegionName),
}

impl<'env> Liveness<'env> {
    pub fn new(env: &'env Environment<'env>) -> Liveness<'env> {
        let bits: Vec<_> = {
            let used_bits = env
                .graph
                .decls()
                .iter()
                .cloned()
                .map(|d| BitKind::VariableUsed(d.var));
            let drop_bits = env
                .graph
                .decls()
                .iter()
                .cloned()
                .map(|d| BitKind::VariableDrop(d.var));
            let free_region_bits = env
                .graph
                .free_regions()
                .iter()
                .cloned()
                .map(|rd| BitKind::FreeRegion(rd.name));
            used_bits.chain(drop_bits).chain(free_region_bits).collect()
        };

        let bits_map: HashMap<_, _> = bits
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, bk)| (bk, index))
            .collect();

        let liveness = BitSet::new(env.graph.len(), bits.len());
        let mut this = Liveness {
            env,
            bits,
            bits_map,
            liveness,
        };
        this.compute();
        this
    }

    pub fn var_live_on_entry(&self, var_name: repr::Variable, b: BlockIndex) -> bool {
        let bit = self.bits_map[&BitKind::VariableUsed(var_name)];
        self.liveness.bits(b).get(bit)
    }

    pub fn region_live_on_entry(&self, region: repr::RegionName, b: BlockIndex) -> bool {
        self.regions_set(self.liveness.bits(b)).contains(&region)
    }

    pub fn live_regions<'a>(
        &'a self,
        live: BitSlice<'a>,
    ) -> impl Iterator<Item = repr::RegionName> + 'a {
        self.regions_set(live).into_iter()
    }

    fn regions_set(&self, live_bits: BitSlice) -> BTreeSet<repr::RegionName> {
        let mut set = BTreeSet::new();
        for (index, bk) in self.bits.iter().enumerate() {
            if live_bits.get(index) {
                match *bk {
                    BitKind::VariableUsed(v) => Self::use_ty(&mut set, &self.env.var_ty(v)),
                    BitKind::VariableDrop(v) => {
                        Self::drop_ty(&mut set, &self.env.var_ty(v), self.env);
                    }
                    BitKind::FreeRegion(rn) => Self::use_region(&mut set, rn),
                }
            }
        }
        set
    }

    /// Invokes callback once for each action with (A) the point of
    /// the action; (B) the action itself and (C) the set of live
    /// variables on entry to the action.
    pub fn walk(&self, mut callback: impl FnMut(Point, Option<&repr::Action>, BitSlice)) {
        let mut bits = self.liveness.empty_buf();
        for &block in &self.env.reverse_post_order {
            self.simulate_block(&mut bits, block, &mut callback);
        }
    }

    fn compute(&mut self) {
        let mut bits = self.liveness.empty_buf();
        let mut changed = true;
        while changed {
            changed = false;

            for &block in &self.env.reverse_post_order {
                self.simulate_block(&mut bits, block, |_p, _a, _s| ());
                changed |= self.liveness.insert_bits_from_slice(block, bits.as_slice());
            }
        }
    }

    fn simulate_block(
        &self,
        buf: &mut BitBuf,
        block: BlockIndex,
        mut callback: impl FnMut(Point, Option<&repr::Action>, BitSlice),
    ) {
        buf.clear();

        // everything live in a successor is live at the exit of the block
        for succ in self.env.graph.successors(block) {
            buf.set_from(self.liveness.bits(succ));
        }

        // callback for the "goto" point
        callback(self.env.end_point(block), None, buf.as_slice());

        // walk backwards through the actions
        for (index, action) in self
            .env
            .graph
            .block_data(block)
            .actions()
            .iter()
            .enumerate()
            .rev()
        {
            let (def_var, use_var) = action.def_use();

            // anything we write to is no longer live
            for v in def_var {
                buf.kill(self.bits_map[&BitKind::VariableUsed(v)]);
                buf.kill(self.bits_map[&BitKind::VariableDrop(v)]);
            }

            // any variables we read from, we make live
            for v in use_var {
                buf.set(self.bits_map[&BitKind::VariableUsed(v)]);
            }

            // some actions are special
            match &action.kind {
                repr::ActionKind::Drop(path) => {
                    buf.set(self.bits_map[&BitKind::VariableDrop(path.base())]);
                }
                &repr::ActionKind::SkolemizedEnd(name) => {
                    buf.set(self.bits_map[&BitKind::FreeRegion(name)]);
                }
                _ => {}
            }

            let point = Point {
                block,
                action: index,
            };
            callback(point, Some(action), buf.as_slice());
        }
    }

    fn use_ty(buf: &mut BTreeSet<repr::RegionName>, ty: &repr::Ty) {
        for region_name in ty.walk_regions().map(repr::Region::assert_free) {
            Self::use_region(buf, region_name);
        }
    }

    fn use_region(buf: &mut BTreeSet<repr::RegionName>, region_name: repr::RegionName) {
        buf.insert(region_name);
    }

    fn drop_ty(buf: &mut BTreeSet<repr::RegionName>, ty: &repr::Ty, env: &'env Environment<'env>) {
        match ty {
            repr::Ty::Ref(..) | repr::Ty::Mut(..) | repr::Ty::Unit => {
                // Dropping a reference (or `()`) does not require it to be live; it's a no-op.
            }

            repr::Ty::Struct(struct_name, params) => {
                let struct_decl = env.struct_map[struct_name];
                assert_eq!(struct_decl.params.len(), params.len());
                for (param_decl, param) in struct_decl.params.iter().zip(params.iter()) {
                    match param.clone() {
                        repr::TyParam::Region(region) => {
                            if !param_decl.may_dangle {
                                Self::use_region(buf, region.assert_free());
                            }
                        }

                        repr::TyParam::Ty(ty) => {
                            if param_decl.may_dangle {
                                Self::drop_ty(buf, &ty, env);
                            } else {
                                Self::use_ty(buf, &ty);
                            }
                        }
                    }
                }
            }

            repr::Ty::Bound(_) => panic!("drop_ty: unexpected bound type {ty:?}"),
        }
    }
}

pub trait DefUse {
    /// Returns (defs, uses), where `defs` contains variables whose
    /// current value is completely overwritten, and `uses` contains
    /// variables whose current value is used. Note that a variable
    /// may exist in both sets.
    fn def_use(&self) -> (Vec<repr::Variable>, Vec<repr::Variable>);
}

impl DefUse for repr::Action {
    fn def_use(&self) -> (Vec<repr::Variable>, Vec<repr::Variable>) {
        match &self.kind {
            repr::ActionKind::Borrow(p, _name, _, q) => (vec![p.base()], vec![q.base()]),
            repr::ActionKind::Init(a, params) => (
                a.write_def().into_iter().collect(),
                params
                    .iter()
                    .map(|p| p.base())
                    .chain(a.write_use())
                    .collect(),
            ),
            repr::ActionKind::Assign(a, b) => (
                a.write_def().into_iter().collect(),
                once(b.base()).chain(a.write_use()).collect(),
            ),
            repr::ActionKind::Constraint(_c) => (vec![], vec![]),
            repr::ActionKind::Use(v) => (vec![], vec![v.base()]),

            // drop is special; it is not considered a "full use" of
            // the variable that is being dropped
            repr::ActionKind::Drop(..)
            | repr::ActionKind::Noop
            | repr::ActionKind::StorageDead(_)
            | repr::ActionKind::SkolemizedEnd(_) => (vec![], vec![]),
        }
    }
}
