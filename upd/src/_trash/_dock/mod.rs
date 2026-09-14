mod node;
mod node_index;
mod tab;

pub use self::node::{LeafNode, Node, SplitNode, TabIndex};
pub use self::node_index::NodeIndex;
pub use self::tab::{Tab, TabIter};

use bevy::ecs::resource::Resource;

/// Direction in which a new node is created relatively to the parent node at which the split occurs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(missing_docs)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

impl Split {
    /// Returns whether the split is vertical.
    pub const fn is_vertical(self) -> bool {
        matches!(self, Split::Above | Split::Below)
    }

    /// Returns whether the split is horizontal.
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Split::Left | Split::Right)
    }
}

#[derive(Resource)]
pub struct Tree<Tab> {
    nodes: Vec<Node<Tab>>,
    focus: Option<NodeIndex>,
}

impl<Tab> std::ops::Index<NodeIndex> for Tree<Tab> {
    type Output = Node<Tab>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<Tab> std::ops::IndexMut<NodeIndex> for Tree<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<Tab> Tree<Tab> {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            nodes: vec![Node::leaf(tabs)],
            focus: None,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Node<Tab>> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Node<Tab>> {
        self.nodes.iter_mut()
    }

    pub fn right(&mut self, parent: NodeIndex, fraction: f32, tabs: Vec<Tab>) -> [NodeIndex; 2] {
        self.split(parent, Split::Right, fraction, Node::leaf(tabs))
    }

    pub fn left(&mut self, parent: NodeIndex, fraction: f32, tabs: Vec<Tab>) -> [NodeIndex; 2] {
        self.split(parent, Split::Left, fraction, Node::leaf(tabs))
    }

    pub fn below(&mut self, parent: NodeIndex, fraction: f32, tabs: Vec<Tab>) -> [NodeIndex; 2] {
        self.split(parent, Split::Below, fraction, Node::leaf(tabs))
    }

    pub fn above(&mut self, parent: NodeIndex, fraction: f32, tabs: Vec<Tab>) -> [NodeIndex; 2] {
        self.split(parent, Split::Above, fraction, Node::leaf(tabs))
    }

    pub fn split_tabs(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, split, fraction, Node::leaf(tabs))
    }

    pub fn split(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        let old = self[parent].split(split, fraction);
        assert!(old.is_leaf());

        let index = self.nodes.iter().rposition(|n| !n.is_empty()).unwrap_or(0);
        let level = NodeIndex(index).level();
        self.nodes.resize_with(1 << (level + 1), || Node::Empty);

        let index = match split {
            Split::Right | Split::Above => [parent.right(), parent.left()],
            Split::Left | Split::Below => [parent.left(), parent.right()],
        };

        self[index[0]] = old;
        self[index[1]] = new;

        index
    }

    pub fn remove_empty_leaf(&mut self) {
        let mut nodes = self.nodes.iter().enumerate();
        let node = nodes.find_map(|(index, node)| match node {
            Node::Leaf(LeafNode { tabs, .. }) if tabs.is_empty() => Some(NodeIndex(index)),
            _ => None,
        });

        let Some(node) = node else { return };

        let parent = node.parent().unwrap();

        self[parent] = Node::Empty;
        self[node] = Node::Empty;

        let mut level = 0;

        if node.is_left() {
            'left_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_right(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.nodes.len() {
                        break 'left_end;
                    }
                    self.nodes[dst] = std::mem::replace(&mut self.nodes[src], Node::Empty);
                }
                level += 1;
            }
        } else {
            'right_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_left(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.nodes.len() {
                        break 'right_end;
                    }
                    self.nodes[dst] = std::mem::replace(&mut self.nodes[src], Node::Empty);
                }
                level += 1;
            }
        }
    }

    /// Returns an iterator over all tabs in arbitrary order.
    #[inline(always)]
    pub fn tabs(&self) -> TabIter<'_, Tab> {
        TabIter::new(self)
    }

    #[inline]
    pub fn num_tabs(&self) -> usize {
        let map = |node: &Node<Tab>| node.tabs().map_or(0, |tabs| tabs.len());
        self.nodes.iter().map(map).sum::<usize>()
    }
}

impl<Tab> Tree<Tab> {
    /// Pushes a tab to the first `Leaf` it finds or create a new leaf if an `Empty` node is encountered.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        for (index, node) in &mut self.nodes.iter_mut().enumerate() {
            match node {
                Node::Leaf(leaf) => {
                    leaf.append_tab(tab);
                    self.focus = Some(NodeIndex(index));
                    return;
                }
                Node::Empty => {
                    *node = Node::leaf(vec![tab]);
                    self.focus = Some(NodeIndex(index));
                    return;
                }
                _ => (),
            }
        }

        assert!(self.nodes.is_empty());
        self.nodes.push(Node::leaf(vec![tab]));
        self.focus = Some(NodeIndex::root());
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        if self.nodes.is_empty() {
            self.nodes.push(Node::leaf(vec![tab]));
            self.focus = Some(NodeIndex::root());
        } else {
            match self.focus {
                Some(node) => match &mut self[node] {
                    Node::Empty => {
                        self[node] = Node::leaf(vec![tab]);
                        self.focus = Some(node);
                    }
                    Node::Leaf(leaf) => {
                        leaf.append_tab(tab);
                        self.focus = Some(node);
                    }
                    _ => self.push_to_first_leaf(tab),
                },
                None => self.push_to_first_leaf(tab),
            }
        }
    }
}
