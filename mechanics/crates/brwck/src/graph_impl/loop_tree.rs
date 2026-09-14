use super::{Dominators, Graph, NodeVec};
use std::collections::HashSet;

pub struct LoopTree<G: Graph> {
    loop_ids: NodeVec<G, Option<LoopId>>,
    loop_infos: Vec<LoopInfo<G>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoopId {
    index: usize,
}

struct LoopInfo<G: Graph> {
    parent: Option<LoopId>,
    head: G::Node,
    exits: Vec<G::Node>,
}

impl<G: Graph> LoopTree<G> {
    pub fn new(graph: &G) -> Self {
        Self {
            loop_ids: NodeVec::from_fn(graph.len(), |_| None),
            loop_infos: vec![],
        }
    }

    pub fn compute(graph: &G, dominators: &Dominators<G>) -> Self {
        LoopTreeWalk::new(graph, dominators).compute_loop_tree()
    }

    pub fn new_loop(&mut self, head: G::Node) -> LoopId {
        let loop_id = LoopId {
            index: self.loop_infos.len(),
        };
        self.loop_infos.push(LoopInfo {
            parent: None, // will get updated later
            head,
            exits: vec![],
        });
        loop_id
    }

    pub fn set_parent(&mut self, loop_id: LoopId, parent_loop_id: Option<LoopId>) {
        self.loop_infos[loop_id.index].parent = parent_loop_id;
    }

    pub fn parent(&self, loop_id: LoopId) -> Option<LoopId> {
        self.loop_infos[loop_id.index].parent
    }

    pub fn parents(&self, loop_id: LoopId) -> Parents<G> {
        Parents {
            tree: self,
            next_loop_id: self.parent(loop_id),
        }
    }

    pub fn loop_head(&self, loop_id: LoopId) -> G::Node {
        self.loop_infos[loop_id.index].head
    }

    pub fn loop_head_of_node(&self, node: G::Node) -> Option<G::Node> {
        self.loop_id(node).map(|loop_id| self.loop_head(loop_id))
    }

    pub fn loop_exits(&self, loop_id: LoopId) -> &[G::Node] {
        &self.loop_infos[loop_id.index].exits
    }

    pub fn push_loop_exit(&mut self, loop_id: LoopId, exit: G::Node) {
        self.loop_infos[loop_id.index].exits.push(exit);
    }

    pub fn loop_id(&self, node: G::Node) -> Option<LoopId> {
        self.loop_ids[node]
    }

    pub fn set_loop_id(&mut self, node: G::Node, id: Option<LoopId>) {
        self.loop_ids[node] = id;
    }
}

pub struct Parents<'iter, G: Graph + 'iter> {
    tree: &'iter LoopTree<G>,
    next_loop_id: Option<LoopId>,
}

impl<G: Graph> Iterator for Parents<'_, G> {
    type Item = LoopId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(loop_id) = self.next_loop_id {
            self.next_loop_id = self.tree.parent(loop_id);
            Some(loop_id)
        } else {
            None
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
enum NodeState {
    #[default]
    NotYetStarted,
    InProgress(Option<LoopId>),
    FinishedHeadWalk,
    EnqueuedExitWalk,
}

pub struct LoopTreeWalk<'walk, G: Graph + 'walk> {
    graph: &'walk G,
    dominators: &'walk Dominators<G>,
    state: NodeVec<G, NodeState>,
    loop_tree: LoopTree<G>,
}

impl<'walk, G: Graph> LoopTreeWalk<'walk, G> {
    pub fn new(graph: &'walk G, dominators: &'walk Dominators<G>) -> Self {
        Self {
            graph,
            dominators,
            state: NodeVec::from_fn(graph.len(), |_| NodeState::default()),
            loop_tree: LoopTree::new(graph),
        }
    }

    pub fn compute_loop_tree(mut self) -> LoopTree<G> {
        self.head_walk(self.graph.start());
        self.exit_walk(self.graph.start());
        self.loop_tree
    }

    /// First walk: identify loop heads and loop parents. This uses a
    /// variant of Tarjan's SCC algorithm. Basically, we do a
    /// depth-first search. Each time we encounter a backedge, the
    /// target of that backedge is a loop-head, so we make a
    /// corresponding loop, if we haven't done so already. We then track
    /// the set of loops that `node` was able to reach via backedges.
    /// The innermost such loop is the loop-id of `node`, and we then
    /// return the set for use by the predecessor of `node`.
    fn head_walk(&mut self, node: G::Node) -> HashSet<LoopId> {
        assert_eq!(self.state[node], NodeState::NotYetStarted);
        self.state[node] = NodeState::InProgress(None);

        // Walk our successors and collect the set of backedges they
        // reach.
        let mut set = HashSet::new();
        for successor in self.graph.successors(node) {
            match self.state[successor] {
                NodeState::NotYetStarted => set.extend(self.head_walk(successor)),
                NodeState::InProgress(opt_loop_id) => {
                    // Backedge. Successor is a loop-head.
                    if let Some(loop_id) = opt_loop_id {
                        set.insert(loop_id);
                    } else {
                        set.insert(self.promote_to_loop_head(successor));
                    }
                }
                NodeState::FinishedHeadWalk => { /* Cross edge. */ }
                NodeState::EnqueuedExitWalk => unreachable!(),
            }
        }

        self.state[node] = NodeState::FinishedHeadWalk;

        // Assign a loop-id to this node. This will be the innermost
        // loop that we could reach.
        if let Some(loop_id) = self.innermost(&set) {
            self.loop_tree.set_loop_id(node, Some(loop_id));

            // Check if we are the loop head. In that case, we
            // should remove ourselves from the returned set,
            // since our parent in the spanning tree is not a
            // member of this loop.
            let loop_head = self.loop_tree.loop_head(loop_id);
            if node == loop_head {
                set.remove(&loop_id);

                // Now the next-innermost loop is the parent of this loop.
                let parent_loop_id = self.innermost(&set);
                self.loop_tree.set_parent(loop_id, parent_loop_id);
            }
        } else {
            assert!(set.is_empty());
            assert!(self.loop_tree.loop_id(node).is_none()); // all none by default
        }

        set
    }

    fn exit_walk(&mut self, node: G::Node) {
        let mut stack = vec![node];

        assert_eq!(self.state[node], NodeState::FinishedHeadWalk);
        self.state[node] = NodeState::EnqueuedExitWalk;

        while let Some(node) = stack.pop() {
            // For each successor, check what loop they are in. If any of
            // them are in a loop outer to ours -- or not in a loop at all
            // -- those are exits from this inner loop.
            if let Some(loop_id) = self.loop_tree.loop_id(node) {
                for successor in self.graph.successors(node) {
                    self.update_loop_exit(loop_id, successor);
                }
            }

            // Visit our successors.
            for successor in self.graph.successors(node) {
                match self.state[successor] {
                    NodeState::NotYetStarted | NodeState::InProgress(_) => {
                        unreachable!();
                    }
                    NodeState::FinishedHeadWalk => {
                        stack.push(successor);
                        self.state[successor] = NodeState::EnqueuedExitWalk;
                    }
                    NodeState::EnqueuedExitWalk => {}
                }
            }
        }
    }

    fn promote_to_loop_head(&mut self, node: G::Node) -> LoopId {
        assert_eq!(self.state[node], NodeState::InProgress(None));
        let loop_id = self.loop_tree.new_loop(node);
        self.state[node] = NodeState::InProgress(Some(loop_id));
        loop_id
    }

    fn innermost(&self, set: &HashSet<LoopId>) -> Option<LoopId> {
        let mut innermost = None;
        for &loop_id1 in set {
            if let Some(loop_id2) = innermost {
                if self.is_inner_loop_of(loop_id1, loop_id2) {
                    innermost = Some(loop_id1);
                }
            } else {
                innermost = Some(loop_id1);
            }
        }
        innermost
    }

    fn is_inner_loop_of(&self, l1: LoopId, l2: LoopId) -> bool {
        let h1 = self.loop_tree.loop_head(l1);
        let h2 = self.loop_tree.loop_head(l2);
        assert!(h1 != h2);
        if self.dominators.is_dominated_by(h1, h2) {
            true
        } else {
            // These two must have a dominance relationship or else
            // the graph is not reducible.
            assert!(self.dominators.is_dominated_by(h2, h1));
            false
        }
    }

    /// Some node that is in loop `loop_id` has the successor
    /// `successor`. Check if `successor` is not in the loop
    /// `loop_id` and update loop exits appropriately.
    fn update_loop_exit(&mut self, mut loop_id: LoopId, successor: G::Node) {
        if let Some(successor_loop_id) = self.loop_tree.loop_id(successor) {
            // If the successor's loop is an outer-loop of ours,
            // then this is an exit from our loop and all
            // intervening loops.
            if self
                .loop_tree
                .parents(loop_id)
                .any(|p| p == successor_loop_id)
            {
                while loop_id != successor_loop_id {
                    self.loop_tree.push_loop_exit(loop_id, successor);
                    loop_id = self.loop_tree.parent(loop_id).unwrap();
                }
            }
        } else {
            // Successor is not in a loop, so this is an exit from
            // `loop_id` and all of its parents.
            let mut p = Some(loop_id);
            while let Some(l) = p {
                self.loop_tree.push_loop_exit(l, successor);
                p = self.loop_tree.parent(l);
            }
        }
    }
}
