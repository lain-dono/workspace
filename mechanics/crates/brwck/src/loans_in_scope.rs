use crate::env::{Environment, Point};
use crate::graph::{BlockIndex, FuncGraph};
use crate::graph_impl::{BitBuf, BitSet, BitSlice, Graph};
use crate::region::Region;
use crate::regionck::RegionCheck;
use crate::repr;
use std::collections::HashMap;

pub struct LoansInScope<'cx> {
    env: &'cx Environment<'cx>,
    loans: Vec<Loan<'cx>>,
    after_block: BitSet<FuncGraph>,
    loans_by_point: HashMap<Point, usize>,
}

#[derive(Debug)]
pub struct Loan<'cx> {
    pub point: Point,
    pub path: &'cx repr::Path,
    pub kind: repr::BorrowKind,
    pub region: &'cx Region,
}

impl<'cx> LoansInScope<'cx> {
    pub fn new(regionck: &'cx RegionCheck<'cx>) -> Self {
        let env = regionck.env();

        // Collect the full set of loans; these are just the set of `&foo` expressions.
        let loans = env.reverse_post_order.iter().flat_map(|&block| {
            env.graph
                .block_data(block)
                .actions()
                .iter()
                .enumerate()
                .filter_map(move |(action, a)| match a.kind {
                    repr::ActionKind::Borrow(_, region, kind, ref path) => Some(Loan {
                        point: Point { block, action },
                        path,
                        kind,
                        region: regionck.region(region),
                    }),
                    _ => None,
                })
        });

        let loans: Vec<_> = loans.collect();

        println!("loans: {loans:#?}");

        // Make a convenient hash map for getting the index of a loan
        // based on where it appears.
        let loans_by_point: HashMap<_, _> = loans
            .iter()
            .enumerate()
            .map(|(index, loan)| (loan.point, index))
            .collect();

        // Get a bit set with the set of in-scope loans at each point
        // in the graph. These correspond to the set of loans in scope
        // at the end of the block.
        let after_block = BitSet::new(env.graph.len(), loans.len());

        // iterate until fixed point
        let mut this = Self {
            env,
            loans,
            after_block,
            loans_by_point,
        };

        this.compute();

        this
    }

    /// Invokes `callback` with the loans in scope at each point.
    pub fn walk(
        &self,
        env: &Environment<'cx>,
        mut callback: impl FnMut(Point, Option<&repr::Action>, &[&Loan]),
    ) {
        let mut loans = Vec::with_capacity(self.loans.len());
        let mut bits = self.after_block.empty_buf();
        for &block in &env.reverse_post_order {
            self.simulate_block(&mut bits, block, |point, action, bits| {
                // Convert from the bitset into a vector of references to loans.
                loans.clear();
                loans.extend(
                    self.loans
                        .iter()
                        .enumerate()
                        .filter_map(|(loan_index, loan)| {
                            if bits.get(loan_index) {
                                Some(loan)
                            } else {
                                None
                            }
                        }),
                );

                // Invoke the callback.
                callback(point, action, &loans);
            });
        }
    }

    /// Iterates until a fixed point, computing the loans in scope
    /// after each block terminates.
    fn compute(&mut self) {
        let mut bits = self.after_block.empty_buf();
        let mut changed = true;
        while changed {
            changed = false;

            for &block in &self.env.reverse_post_order {
                self.simulate_block(&mut bits, block, |_p, _a, _s| ());
                changed |= self
                    .after_block
                    .insert_bits_from_slice(block, bits.as_slice());
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

        // everything live at end of a pred  is live at the exit of the block
        for succ in self.env.graph.predecessors(block) {
            buf.set_from(self.after_block.bits(succ));
        }

        // walk through the actions on by one
        for (index, action) in self
            .env
            .graph
            .block_data(block)
            .actions()
            .iter()
            .enumerate()
        {
            let point = Point {
                block,
                action: index,
            };

            // kill any loans where `point` is not in their region
            for loan_index in self.loans_not_in_scope_at(point) {
                buf.kill(loan_index);
            }

            // callback at start of the action
            callback(point, Some(action), buf.as_slice());

            // bring the loan into scope after the borrow
            if let Some(&loan_index) = self.loans_by_point.get(&point) {
                buf.set(loan_index);
            }

            // figure out which path is overwritten by this action;
            // this may cancel out some loans
            if let Some(overwritten_path) = action.overwrites() {
                for loan_index in self.loans_killed_by_write_to(overwritten_path) {
                    buf.kill(loan_index);
                }
            }
        }

        // final callback for the terminator
        let point = self.env.end_point(block);
        for loan_index in self.loans_not_in_scope_at(point) {
            buf.kill(loan_index);
        }
        callback(point, None, buf.as_slice());
    }

    fn loans_not_in_scope_at(&self, point: Point) -> impl Iterator<Item = usize> + '_ {
        self.loans
            .iter()
            .enumerate()
            .filter_map(move |(loan_index, loan)| {
                if loan.region.may_contain(point) {
                    None
                } else {
                    Some(loan_index)
                }
            })
    }

    fn loans_killed_by_write_to<'a>(
        &'a self,
        path: &'a repr::Path,
    ) -> impl Iterator<Item = usize> + 'a {
        // When an assignment like `a.b.c = ...` occurs, we kill all
        // the loans for `a.b.c` or some subpath like `a.b.c.d`, since
        // the path no longer evaluates to the same thing.
        self.loans
            .iter()
            .enumerate()
            .filter_map(move |(index, loan)| {
                if loan.path.prefixes().contains(&path) {
                    Some(index)
                } else {
                    None
                }
            })
    }
}

impl repr::Action {
    /// Returns path that this action overwrites, if any.
    fn overwrites(&self) -> Option<&repr::Path> {
        match &self.kind {
            repr::ActionKind::Borrow(path, _, _, _)
            | repr::ActionKind::Init(path, _)
            | repr::ActionKind::Assign(path, _) => Some(path),

            repr::ActionKind::Constraint(_)
            | repr::ActionKind::Use(_)
            | repr::ActionKind::Drop(_)
            | repr::ActionKind::Noop
            | repr::ActionKind::SkolemizedEnd(_)
            | repr::ActionKind::StorageDead(_) => None,
        }
    }
}
