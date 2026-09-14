use super::{LinkInfo, Node};
use std::num::NonZeroU32;

pub(super) type NodeIndex = NonZeroU32;

pub(super) struct Nodes<I> {
    pub(super) data: Vec<Node<I>>,
}

impl<I: Default> Default for Nodes<I> {
    fn default() -> Self {
        Self { data: vec![] }
    }
}

impl<I> std::ops::Index<NodeIndex> for Nodes<I> {
    type Output = Node<I>;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        debug_assert!(index.get() < self.data.len() as u32);
        unsafe { self.data.get_unchecked(index.get() as usize) }
    }
}

impl<I> std::ops::IndexMut<NodeIndex> for Nodes<I> {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        debug_assert!(index.get() < self.data.len() as u32);
        unsafe { self.data.get_unchecked_mut(index.get() as usize) }
    }
}

impl<I> Nodes<I> {
    pub(super) fn push(&mut self, node: Node<I>) {
        self.data.push(node);
    }

    /// create a node and optionally link it with previous one (in a circular doubly linked list)
    pub(super) fn insert(
        &mut self,
        index: I,
        xy: [f32; 2],
        last_index: Option<NodeIndex>,
    ) -> NodeIndex {
        let mut node = Node::new(index, xy);
        let index = unsafe { NodeIndex::new_unchecked(self.data.len() as u32) };
        match last_index {
            Some(last_index) => {
                let last = &mut self[last_index];
                let last_next = last.next;
                (node.next, last.next) = (last_next, index);
                node.prev = last_index;
                self[last_next].prev = index;
            }
            None => {
                (node.prev, node.next) = (index, index);
            }
        }
        self.data.push(node);
        index
    }

    pub fn remove(&mut self, info: LinkInfo) -> (NodeIndex, NodeIndex) {
        let prev = &mut self[info.prev];
        prev.next = info.next;

        if let Some(prev_z) = info.prev_z {
            if prev_z == info.prev {
                prev.next_z = info.next_z;
            } else {
                self[prev_z].next_z = info.next_z;
            }
        }

        let next = &mut self[info.next];
        next.prev = info.prev;

        if let Some(next_z_i) = info.next_z {
            if next_z_i == info.next {
                next.prev_z = info.prev_z;
            } else {
                self[next_z_i].prev_z = info.prev_z;
            }
        }

        (info.prev, info.next)
    }
}
