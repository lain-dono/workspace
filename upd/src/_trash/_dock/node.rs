use super::Split;
use bevy_egui::egui::Rect;

/// Identifies a tab within a [`Node`](crate::Node).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TabIndex(pub usize);

impl From<usize> for TabIndex {
    #[inline]
    fn from(index: usize) -> Self {
        Self(index)
    }
}

/// The inner data of a [``Node::Leaf``](crate::Node), which contains tabs and can be collapsed.
#[derive(Clone, Debug)]
pub struct LeafNode<Tab> {
    /// All the tabs in this node.
    pub tabs: Vec<Tab>,
    /// The full rectangle - tab bar plus tab body.
    pub rect: Rect,
    /// The tab body rectangle.
    pub viewport: Rect,
    /// The opened tab.
    pub active: TabIndex,
    /// Scroll amount of the tab bar.
    pub scroll: f32,
    /// Whether the leaf is collapsed.
    pub collapsed: bool,
}

impl<Tab> LeafNode<Tab> {
    /// Create New LeafNode with specified ``tabs``, all other internal values will be filled by "nothing" defaults.
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            tabs,
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            active: TabIndex(0),
            scroll: 0.0,
            collapsed: false,
        }
    }

    /// Append a ``Tab`` to the end of this [`LeafNode`]s tab list.
    ///
    /// This will also focus the added tab.
    #[track_caller]
    #[inline]
    pub fn append_tab(&mut self, tab: Tab) {
        self.active = TabIndex(self.tabs.len());
        self.tabs.push(tab);
    }

    /// Insert a ``Tab`` to this [`LeafNode`]s tab list at the specified [`TabIndex`].
    ///
    /// This will also focus the added tab.
    ///
    /// # Panics
    ///
    /// if ``index`` exceeds the leaf's tab list length.
    #[track_caller]
    #[inline]
    pub fn insert_tab(&mut self, index: impl Into<TabIndex>, tab: Tab) {
        let index = index.into();
        self.tabs.insert(index.0, tab);
        self.active = index;
    }

    /// Remove a ``Tab`` to this [`LeafNode`]s tab list at the specified [`TabIndex`].
    ///
    /// This will also focus the added tab.'
    ///
    /// # Panics
    ///
    /// if ``index`` is out of bounds for the tab list
    #[inline]
    pub fn remove_tab(&mut self, index: impl Into<TabIndex>) -> Option<Tab> {
        let index = index.into();
        if index <= self.active {
            self.active.0 = self.active.0.saturating_sub(1);
        }
        Some(self.tabs.remove(index.0))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    pub fn retain_tabs<F>(&mut self, predicate: impl FnMut(&mut Tab) -> bool) {
        self.tabs.retain_mut(predicate);
    }

    /// Return the area and tab which is currently representing this [`LeafNode`]
    ///
    /// This may return ``None`` if the leaf contains 0 tabs.
    #[inline]
    pub fn active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        let TabIndex(index) = self.active;
        self.tabs.get_mut(index).map(|tab| (self.viewport, tab))
    }
}

/// The inner data of a [``Node::Horizontal``](crate::Node)/[``Node::Vertical``](crate::Node), which splits into two further nodes.
#[derive(Clone, Debug)]
pub struct SplitNode {
    /// The rectangle in which all children of this node are drawn.
    pub rect: Rect,
    /// The fraction taken by the top child of this node.
    pub fraction: f32,
    /// Whether all subnodes are collapsed.
    pub fully_collapsed: bool,
    /// The number of collapsed leaf subnodes.
    pub collapsed_leaf_count: i32,
}

impl SplitNode {
    /// Create a new ``SplitNode``
    pub const fn new(
        rect: Rect,
        fraction: f32,
        fully_collapsed: bool,
        collapsed_leaf_count: i32,
    ) -> Self {
        Self {
            rect,
            fraction,
            fully_collapsed,
            collapsed_leaf_count,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Node<Tab> {
    /// Empty node.
    Empty,
    /// Contains the actual tabs.
    Leaf(LeafNode<Tab>),
    /// Parent node in the vertical orientation.
    Vertical(SplitNode),
    /// Parent node in the horizontal orientation.
    Horizontal(SplitNode),
}

impl<Tab> Node<Tab> {
    pub fn leaf(tabs: Vec<Tab>) -> Self {
        Self::Leaf(LeafNode::new(tabs))
    }

    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::Empty => (),
            Self::Leaf(leaf) => leaf.rect = new_rect,
            Self::Vertical(split) | Self::Horizontal(split) => split.rect = new_rect,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf(..))
    }

    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        let node = SplitNode::new(Rect::NOTHING, fraction, false, 0);
        let src = match split {
            Split::Left | Split::Right => Self::Horizontal(node),
            Split::Above | Split::Below => Self::Vertical(node),
        };
        std::mem::replace(self, src)
    }

    pub fn tabs(&self) -> Option<&[Tab]> {
        match self {
            Node::Leaf(leaf) => Some(&leaf.tabs),
            _ => None,
        }
    }

    pub fn tabs_mut(&mut self) -> Option<&mut [Tab]> {
        match self {
            Node::Leaf(leaf) => Some(&mut leaf.tabs),
            _ => None,
        }
    }

    pub fn append_tab(&mut self, tab: Tab) {
        let Node::Leaf(leaf) = self else {
            unreachable!()
        };
        leaf.append_tab(tab);
    }

    pub fn remove_tab(&mut self, index: impl Into<TabIndex>) -> Option<Tab> {
        let Node::Leaf(leaf) = self else { return None };
        leaf.remove_tab(index)
    }
}
