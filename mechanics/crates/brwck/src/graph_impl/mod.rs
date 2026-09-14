mod bitset;
mod dominator;
mod loop_tree;

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[cfg(test)]
mod tests;

pub use self::bitset::{BitBuf, BitSet, BitSlice};
pub use self::dominator::{DominatorTree, Dominators};
pub use self::loop_tree::LoopTree;

pub trait Graph: Sized {
    type Node: Copy + Debug + Eq + Ord + Hash + Into<usize> + From<usize>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn start(&self) -> Self::Node;

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node>;
    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node>;

    fn post_order(&self, start: Self::Node, end: Option<Self::Node>) -> Vec<Self::Node> {
        let mut visited = NodeVec::from_fn(self.len(), |_| false);
        let mut result = Vec::with_capacity(self.len());
        if let Some(end) = end {
            visited[end] = true;
        }
        self.post_order_walk(start, &mut result, &mut visited);
        result
    }

    fn post_order_walk(
        &self,
        node: Self::Node,
        result: &mut Vec<Self::Node>,
        visited: &mut NodeVec<Self, bool>,
    ) {
        if !visited[node] {
            visited[node] = true;
            for successor in self.successors(node) {
                Self::post_order_walk(self, successor, result, visited);
            }
            result.push(node);
        }
    }
}

pub struct NodeVec<G: Graph, T> {
    inner: Vec<T>,
    graph: PhantomData<G>,
}

impl<G: Graph, T> NodeVec<G, T> {
    pub fn from_fn(len: usize, f: impl FnMut(G::Node) -> T) -> Self {
        Self {
            inner: (0..len).map(G::Node::from).map(f).collect(),
            graph: PhantomData,
        }
    }
}

impl<G: Graph, T> Deref for NodeVec<G, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<G: Graph, T> DerefMut for NodeVec<G, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<G: Graph, T> Index<G::Node> for NodeVec<G, T> {
    type Output = T;

    fn index(&self, index: G::Node) -> &T {
        &self.inner[index.into()]
    }
}

impl<G: Graph, T> IndexMut<G::Node> for NodeVec<G, T> {
    fn index_mut(&mut self, index: G::Node) -> &mut T {
        &mut self.inner[index.into()]
    }
}

pub struct TransposedGraph<G: Graph> {
    base: G,
    start: G::Node,
}

impl<G: Graph> TransposedGraph<G> {
    pub fn new(base_graph: G) -> Self {
        let start_node = base_graph.start();
        Self::with_start(base_graph, start_node)
    }

    pub fn with_start(base: G, start: G::Node) -> Self {
        Self { base, start }
    }
}

impl<G: Graph> Graph for TransposedGraph<G> {
    type Node = G::Node;

    fn len(&self) -> usize {
        self.base.len()
    }

    fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    fn start(&self) -> Self::Node {
        self.start
    }

    fn predecessors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> {
        self.base.successors(node)
    }

    fn successors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> {
        self.base.predecessors(node)
    }
}

pub struct Reachability<G: Graph> {
    bits: BitSet<G>,
}

impl<G: Graph> Reachability<G> {
    fn new(num_nodes: usize) -> Self {
        Self {
            bits: BitSet::new(num_nodes, num_nodes),
        }
    }

    pub fn can_reach(&self, source: G::Node, target: G::Node) -> bool {
        self.bits.is_set(source, target.into())
    }

    // Compute reachability using a simple dataflow propagation.
    // Store end-result in a big NxN bit matrix.
    pub fn reachable(graph: &G) -> Self {
        let mut reverse_post_order = graph.post_order(graph.start(), None);
        reverse_post_order.reverse();
        Self::reachable_with(graph, &reverse_post_order)
    }

    pub fn reachable_with(graph: &G, reverse_post_order: &[G::Node]) -> Self {
        let mut reachability = Reachability::new(graph.len());
        let mut changed = true;
        while changed {
            changed = false;

            for &node in reverse_post_order.iter().rev() {
                // every node can reach itself
                changed |= reachability.bits.insert(node, node.into());
                // and every pred can reach everything node can reach
                for pred in graph.predecessors(node) {
                    changed |= reachability.bits.insert_bits_from_node(node, pred);
                }
            }
        }
        reachability
    }
}
