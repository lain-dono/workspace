use super::{Dominators, Graph, LoopTree, Reachability, TransposedGraph};
use std::cmp::max;
use std::collections::HashMap;

fn loop_tree<G: Graph>(graph: &G) -> LoopTree<G> {
    LoopTree::compute(graph, &Dominators::new(graph))
}

pub struct TestGraph {
    num_nodes: usize,
    start_node: usize,
    successors: HashMap<usize, Vec<usize>>,
    predecessors: HashMap<usize, Vec<usize>>,
}

impl TestGraph {
    pub fn new(start_node: usize, edges: &[(usize, usize)]) -> Self {
        let mut graph = TestGraph {
            num_nodes: start_node + 1,
            start_node,
            successors: HashMap::new(),
            predecessors: HashMap::new(),
        };
        for &(source, target) in edges {
            graph.num_nodes = max(graph.num_nodes, source + 1);
            graph.num_nodes = max(graph.num_nodes, target + 1);
            graph.successors.entry(source).or_default().push(target);
            graph.predecessors.entry(target).or_default().push(source);
        }
        for node in 0..graph.num_nodes {
            graph.successors.entry(node).or_default();
            graph.predecessors.entry(node).or_default();
        }
        graph
    }
}

impl Graph for TestGraph {
    type Node = usize;

    fn start(&self) -> usize {
        self.start_node
    }

    fn len(&self) -> usize {
        self.num_nodes
    }

    fn is_empty(&self) -> bool {
        self.num_nodes == 0
    }

    fn predecessors(&self, node: usize) -> impl Iterator<Item = Self::Node> {
        self.predecessors[&node].iter().copied()
    }

    fn successors(&self, node: usize) -> impl Iterator<Item = Self::Node> {
        self.successors[&node].iter().copied()
    }
}

#[test]
fn diamond_post_order() {
    let graph = TestGraph::new(0, &[(0, 1), (0, 2), (1, 3), (2, 3)]);

    let result = graph.post_order(0, None);
    assert_eq!(result, vec![3, 1, 2, 0]);
}

#[test]
fn rev_post_order_inner_loop() {
    // 0 -> 1 ->     2     -> 3 -> 5
    //      ^     ^    v      |
    //      |     6 <- 4      |
    //      +-----------------+
    let graph = TestGraph::new(
        0,
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 5),
            (3, 1),
            (2, 4),
            (4, 6),
            (6, 2),
        ],
    );

    let rev_graph = TransposedGraph::new(graph);

    let result = rev_graph.post_order(6, Some(2));
    assert_eq!(result, vec![4, 6]);

    let result = rev_graph.post_order(3, Some(1));
    assert_eq!(result, vec![4, 6, 2, 3]);
}

#[test]
fn reachable_test1() {
    // 0 -> 1 -> 2 -> 3
    //      ^    v
    //      6 <- 4 -> 5
    let graph = TestGraph::new(0, &[(0, 1), (1, 2), (2, 3), (2, 4), (4, 5), (4, 6), (6, 1)]);
    let reachable = Reachability::reachable(&graph);
    assert!((0..6).all(|i| reachable.can_reach(0, i)));
    assert!((1..6).all(|i| reachable.can_reach(1, i)));
    assert!((1..6).all(|i| reachable.can_reach(2, i)));
    assert!((1..6).all(|i| reachable.can_reach(4, i)));
    assert!((1..6).all(|i| reachable.can_reach(6, i)));
    assert!(reachable.can_reach(3, 3));
    assert!(!reachable.can_reach(3, 5));
    assert!(!reachable.can_reach(5, 3));
}

/// use bigger indices to cross between words in the bit set
#[test]
fn reachable_test2() {
    // 30 -> 31 -> 32 -> 33
    //       ^      v
    //       36 <- 34 -> 35
    let graph = TestGraph::new(
        30,
        &[
            (30, 31),
            (31, 32),
            (32, 33),
            (32, 34),
            (34, 35),
            (34, 36),
            (36, 31),
        ],
    );

    let reachable = Reachability::reachable(&graph);
    assert!((30..36).all(|i| reachable.can_reach(30, i)));
    assert!((31..36).all(|i| reachable.can_reach(31, i)));
    assert!((31..36).all(|i| reachable.can_reach(32, i)));
    assert!((31..36).all(|i| reachable.can_reach(34, i)));
    assert!((31..36).all(|i| reachable.can_reach(36, i)));
    assert!(reachable.can_reach(33, 33));
    assert!(!reachable.can_reach(33, 35));
    assert!(!reachable.can_reach(35, 33));
}

#[test]
fn dominator_diamond() {
    let graph = TestGraph::new(0, &[(0, 1), (0, 2), (1, 3), (2, 3)]);

    let dominators = Dominators::new(&graph);
    assert_eq!(
        &dominators.all_immediate_dominators().inner[..],
        &[Some(0), Some(0), Some(0), Some(0)]
    );
}

#[test]
fn dominator_paper() {
    // example from the paper:
    let graph = TestGraph::new(
        6,
        &[
            (6, 5),
            (6, 4),
            (5, 1),
            (4, 2),
            (4, 3),
            (1, 2),
            (2, 3),
            (3, 2),
            (2, 1),
        ],
    );

    let dominators = Dominators::new(&graph);
    assert_eq!(
        &dominators.all_immediate_dominators().inner[..],
        &[
            None, // <-- note that 0 is not in graph
            Some(6),
            Some(6),
            Some(6),
            Some(6),
            Some(6),
            Some(6)
        ]
    );
}

#[test]
fn loop_test1() {
    // 0 -> 1 -> 2 -> 3
    //      ^    v
    //      6 <- 4 -> 5
    let graph = TestGraph::new(0, &[(0, 1), (1, 2), (2, 3), (2, 4), (4, 5), (4, 6), (6, 1)]);
    let loop_tree = loop_tree(&graph);
    assert_eq!(loop_tree.loop_head_of_node(0), None);
    assert_eq!(loop_tree.loop_head_of_node(1), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(2), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(3), None);
    assert_eq!(loop_tree.loop_head_of_node(4), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(5), None);
    assert_eq!(loop_tree.loop_head_of_node(6), Some(1));

    let loop_id = loop_tree.loop_id(1).unwrap();
    assert_eq!(loop_tree.loop_id(2), Some(loop_id));
    assert_eq!(loop_tree.parent(loop_id), None);
    assert_eq!(loop_tree.loop_exits(loop_id), &[3, 5]);
}

#[test]
fn nested_loop() {
    // 0 -> 1 ->     2     -> 3 -> 5
    //      ^     ^    v      |
    //      |     6 <- 4      |
    //      +-----------------+
    let graph = TestGraph::new(
        0,
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 5),
            (3, 1),
            (2, 4),
            (4, 6),
            (6, 2),
        ],
    );
    let loop_tree = loop_tree(&graph);
    assert_eq!(loop_tree.loop_head_of_node(0), None);
    assert_eq!(loop_tree.loop_head_of_node(1), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(2), Some(2));
    assert_eq!(loop_tree.loop_head_of_node(3), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(4), Some(2));
    assert_eq!(loop_tree.loop_head_of_node(5), None);
    assert_eq!(loop_tree.loop_head_of_node(6), Some(2));

    let outer_loop_id = loop_tree.loop_id(1).unwrap();
    let inner_loop_id = loop_tree.loop_id(2).unwrap();
    assert_eq!(loop_tree.parent(outer_loop_id), None);
    assert_eq!(loop_tree.parent(inner_loop_id), Some(outer_loop_id));

    assert_eq!(loop_tree.loop_exits(outer_loop_id), &[5]);
    assert_eq!(loop_tree.loop_exits(inner_loop_id), &[3]);
}

#[test]
fn if_else_break_nested_loop() {
    // 0 -> 1 ->     2     -> 3 -> 5
    //      ^     ^    v      |    ^
    //      |     6 <- 4      |    |
    //      |          |      |    |
    //      |     7 <--+      |    |
    //      +-----|-----------+    |
    //            +----------------+
    let graph = TestGraph::new(
        0,
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 5),
            (3, 1),
            (2, 4),
            (4, 6),
            (4, 7),
            (6, 2),
            (7, 5),
        ],
    );
    let loop_tree = loop_tree(&graph);
    assert_eq!(loop_tree.loop_head_of_node(0), None);
    assert_eq!(loop_tree.loop_head_of_node(1), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(2), Some(2));
    assert_eq!(loop_tree.loop_head_of_node(3), Some(1));
    assert_eq!(loop_tree.loop_head_of_node(4), Some(2));
    assert_eq!(loop_tree.loop_head_of_node(5), None);
    assert_eq!(loop_tree.loop_head_of_node(6), Some(2));
    assert_eq!(loop_tree.loop_head_of_node(7), None);

    let outer_loop_id = loop_tree.loop_id(1).unwrap();
    let inner_loop_id = loop_tree.loop_id(2).unwrap();
    assert_eq!(loop_tree.parent(outer_loop_id), None);
    assert_eq!(loop_tree.parent(inner_loop_id), Some(outer_loop_id));

    assert_eq!(loop_tree.loop_exits(outer_loop_id), &[7, 5]);
    assert_eq!(loop_tree.loop_exits(inner_loop_id), &[3, 7]);
}

#[test]
fn wacked() {
    // This example looks kind of mind-bending,
    // but really isn't. It could result from some code like:
    //
    //     loop {
    //         if ... {
    //             continue
    //         } else {
    //             continue
    //         }
    //     }
    //
    // It came from Munchnick's book.
    //
    // +-1
    // v/
    // 0--->3
    // ^\
    // +-2
    let graph = TestGraph::new(0, &[(0, 1), (1, 0), (0, 2), (2, 0), (0, 3)]);
    let loop_tree = loop_tree(&graph);
    assert_eq!(loop_tree.loop_head_of_node(0), Some(0));
    assert_eq!(loop_tree.loop_head_of_node(1), Some(0));
    assert_eq!(loop_tree.loop_head_of_node(2), Some(0));
    assert_eq!(loop_tree.loop_head_of_node(3), None);

    let outer_loop_id = loop_tree.loop_id(0).unwrap();
    assert_eq!(loop_tree.loop_exits(outer_loop_id), &[3]);
}
