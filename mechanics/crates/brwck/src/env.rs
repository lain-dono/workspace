use crate::graph::{BlockIndex, FuncGraph};
use crate::graph_impl::{DominatorTree, Dominators, Graph, LoopTree, Reachability};
use crate::repr;
use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub block: BlockIndex,
    pub action: usize,
}

impl fmt::Debug for Point {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}/{}", self.block, self.action)
    }
}

pub struct Environment<'func> {
    pub graph: &'func FuncGraph,
    pub dominators: Dominators<FuncGraph>,
    pub dominator_tree: DominatorTree<FuncGraph>,
    pub reachable: Reachability<FuncGraph>,
    pub loop_tree: LoopTree<FuncGraph>,
    pub reverse_post_order: Vec<BlockIndex>,
    pub var_map: HashMap<repr::Variable, &'func repr::VariableDecl>,
    pub struct_map: HashMap<repr::StructName, &'func repr::StructDecl>,
}

impl<'func> Environment<'func> {
    pub fn new(graph: &'func FuncGraph) -> Self {
        let mut rpo = graph.post_order(graph.start(), None);
        rpo.reverse();

        let dominators = Dominators::with_rpo(graph, &rpo);
        let dominator_tree = dominators.dominator_tree();
        let reachable = Reachability::reachable_with(graph, &rpo);
        let loop_tree = LoopTree::compute(graph, &dominators);
        let var_map = graph.decls().iter().map(|vd| (vd.var, vd)).collect();

        let struct_map = graph
            .struct_decls()
            .iter()
            .map(|sd| (sd.name, sd))
            .collect();

        Self {
            graph,
            dominators,
            dominator_tree,
            reachable,
            loop_tree,
            reverse_post_order: rpo,
            var_map,
            struct_map,
        }
    }

    pub fn dump_dominators(&self) {
        fn dump_dominator_tree<G1: Graph<Node = BlockIndex>>(
            tree: &DominatorTree<G1>,
            node: BlockIndex,
            indent: usize,
        ) {
            println!("{0:1$}- {2:?}", "", indent, node);

            for &child in tree.children(node) {
                dump_dominator_tree(tree, child, indent + 2);
            }
        }

        let tree = self.dominators.dominator_tree();
        dump_dominator_tree(&tree, tree.root(), 0);
    }

    pub fn start_point(&self, block: BlockIndex) -> Point {
        Point { block, action: 0 }
    }

    pub fn end_point(&self, block: BlockIndex) -> Point {
        let action = self.graph.block_data(block).actions().len();
        Point { block, action }
    }

    pub fn successor_points(&self, p: Point) -> Vec<Point> {
        let end_point = self.end_point(p.block);
        if p == end_point {
            self.graph
                .successors(p.block)
                .map(|b| self.start_point(b))
                .collect()
        } else {
            let block = p.block;
            let action = p.action + 1;
            vec![Point { block, action }]
        }
    }

    pub fn var_ty(&self, v: repr::Variable) -> repr::Ty {
        match self.var_map.get(&v) {
            Some(decl) => decl.ty.clone(),
            None => panic!("no variable named {v:?}"),
        }
    }

    pub fn path_ty(&self, path: &repr::Path) -> repr::Ty {
        match *path {
            repr::Path::Var(v) => self.var_ty(v),
            repr::Path::Extension(ref base, field_name) => {
                let base_ty = self.path_ty(base);
                self.field_ty(&base_ty, field_name)
            }
        }
    }

    pub fn field_ty(&self, base_ty: &repr::Ty, field_name: repr::FieldName) -> repr::Ty {
        println!("field_ty(base_ty={base_ty:?} field_name={field_name:?})");

        match base_ty {
            repr::Ty::Ref(_, ty) | repr::Ty::Mut(_, ty) => {
                if field_name == repr::FieldName::STAR {
                    *ty.clone()
                } else {
                    panic!("cannot index & with field `{field_name:?}`, use `star`")
                }
            }

            repr::Ty::Unit => panic!("cannot index `()` type"),

            repr::Ty::Struct(n, parameters) => {
                let struct_decl = self.struct_map[n];
                let field_decl = struct_decl
                    .fields
                    .iter()
                    .find(|fd| fd.name == field_name)
                    .unwrap_or_else(|| panic!("no field named `{field_name:?}` in `{n:?}`"));

                let field_ty = &field_decl.ty;
                println!("field_ty: field_ty={field_ty:?} parameters={parameters:?}");

                let field_ty = field_ty.subst(parameters);
                println!("field_ty: field_ty={field_ty:?} post-substitution");

                field_ty
            }

            repr::Ty::Bound(_) => panic!("field_ty: unexpected bound type"),
        }
    }

    /// The **supporting prefixes** of a path are all the prefixes of
    /// a path that must remain valid for the path itself to remain
    /// valid. For the most part, this means all prefixes, except that
    /// recursion stops when dereferencing a shared reference.
    ///
    /// Examples:
    ///
    /// - the supporting prefixes of `s.f` where `s` is a struct are
    ///   `s.f` and `s`.
    /// - the supporting prefixes of `(*r).f` where `r` is a shared reference
    ///   are `(*r).f` and `*r`, but not `r`.
    ///   - Intuition: one could always copy `*r` into a temporary `t`
    ///     and reach the data through `*t`, so it is not important to
    ///     preserve `r` itself.
    /// - the supporting prefixes of `(*m).f` where `m` is a **mutable** reference
    ///   are `(*m).f`, `*m`, and `m`.
    ///
    /// Uses: Supporting prefixes appear in a number of places in the NLL
    /// prototype:
    ///
    /// - the regionck adds sufficient constraints to ensure that the lifetime
    ///   of any reference `r` where `*r` supports a borrowed path outlives
    ///   the lifetime of the borrow (and hence `*r` remains valid).
    /// - the borrowck prevents moves from supporting paths, and prevents reads
    ///   from supporting paths of mutable borrows
    ///
    /// (The mutation and `StorageDead` rules however do not use
    /// supporting prefixes, but rather a further subset.)
    pub fn supporting_prefixes<'a>(&self, mut path: &'a repr::Path) -> Vec<&'a repr::Path> {
        let mut result = vec![];
        loop {
            result.push(path);
            match path {
                repr::Path::Var(_) => return result,
                repr::Path::Extension(base_path, field_name) => {
                    match self.path_ty(base_path) {
                        // If you borrowed `*r`, and `r` is a shared
                        // reference, then accessing `r` (or some
                        // prefix of `r`) is not considered
                        // intersecting. This is because we could have
                        // copied the shared reference out and
                        // borrowed from there.
                        //
                        // This is crucial to a number of tests, e.g.:
                        //
                        // borrowck-write-variable-after-ref-extracted.nll
                        repr::Ty::Ref(..) => {
                            assert_eq!(field_name, &repr::FieldName::STAR);
                            return result;
                        }

                        // In contrast, if you have borrowed `*r`, and
                        // `r` is an `&mut` reference, then we
                        // consider access to `r` intersecting.
                        //
                        // This is crucial to a number of tests, e.g.:
                        //
                        // borrowck-read-ref-while-referent-mutably-borrowed.nll
                        repr::Ty::Mut(..) => path = base_path,

                        // If you have borrowed `a.b`, then writing to
                        // `a` would overwrite `a.b`, which is
                        // disallowed.
                        repr::Ty::Struct(..) => path = base_path,

                        repr::Ty::Unit => panic!("unit has no fields"),
                        repr::Ty::Bound(..) => panic!("unexpected bound type"),
                    }
                }
            }
        }
    }
}
