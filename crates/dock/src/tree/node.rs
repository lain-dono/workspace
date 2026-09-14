use super::TabIndex;
use egui::Rect;

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

///the inner data of a [``Node::Horizontal``](crate::Node)/[``Node::Vertical``](crate::Node), which splits into two further nodes.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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

/// The inner data of a [``Node::Leaf``](crate::Node), which contains tabs and can be collapsed.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LeafNode<Tab> {
    /// The full rectangle - tab bar plus tab body.
    pub rect: Rect,
    /// The tab body rectangle.
    pub viewport: Rect,
    /// All the tabs in this node.
    pub tabs: Vec<Tab>,
    /// The opened tab.
    pub active: TabIndex,
    /// Scroll amount of the tab bar.
    pub scroll: f32,
    /// Whether the leaf is collapsed.
    pub collapsed: bool,
}

/// Represents an abstract node of a [`Tree`](crate::Tree).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
    /// Constructs a leaf node with a given list of `tabs`.
    #[inline(always)]
    pub const fn new(tabs: Vec<Tab>) -> Self {
        Self::Leaf(LeafNode {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: TabIndex(0),
            scroll: 0.0,
            collapsed: false,
        })
    }

    /// Get immutable access to the leaf data of this node, if it contains any (i.e is a leaf)
    pub fn leaf(&self) -> Option<&LeafNode<Tab>> {
        match self {
            Self::Leaf(leaf_node) => Some(leaf_node),
            _ => None,
        }
    }

    /// Get mutable access to the leaf data of this node, if it contains any (i.e is a leaf)
    pub fn leaf_mut(&mut self) -> Option<&mut LeafNode<Tab>> {
        match self {
            Self::Leaf(leaf_node) => Some(leaf_node),
            _ => None,
        }
    }

    /// Get a [`Rect`] occupied by the node, could be used e.g. to draw a highlight rect around a node.
    ///
    /// Returns [`None`] if node is of the [`Empty`](Node::Empty) variant.
    #[inline]
    pub fn rect(&self) -> Option<Rect> {
        match self {
            Self::Empty => None,
            Self::Leaf(leaf) => Some(leaf.rect),
            Self::Vertical(split) | Self::Horizontal(split) => Some(split.rect),
        }
    }

    /// Returns `true` if the node is a [`Empty`](Node::Empty), otherwise `false`.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the node is a [`Leaf`](Node::Leaf), otherwise `false`.
    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Returns `true` if the node is a [`Horizontal`](Node::Horizontal), otherwise `false`.
    #[inline(always)]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal { .. })
    }

    /// Returns `true` if the node is a [`Vertical`](Node::Vertical), otherwise `false`.
    #[inline(always)]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    /// Returns `true` if the node is either [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical),
    /// otherwise `false`.
    #[inline(always)]
    pub const fn is_parent(&self) -> bool {
        self.is_horizontal() || self.is_vertical()
    }

    /// Returns `true` if the node is collapsed, otherwise `false`.
    #[inline(always)]
    pub fn is_collapsed(&self) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.collapsed,
            Self::Horizontal(split) | Self::Vertical(split) => split.fully_collapsed,
            Self::Empty => false,
        }
    }

    /// Returns the number of layers of collapsed leaf subnodes.
    pub fn collapsed_leaf_count(&self) -> i32 {
        match self {
            Self::Horizontal(split) | Self::Vertical(split) => split.collapsed_leaf_count,
            Self::Leaf(leaf) => i32::from(leaf.collapsed),
            Self::Empty => 0,
        }
    }

    /// Replaces the node with [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical) (depending on `split`)
    /// and assigns an empty rect to it.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline]
    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        assert!((0.0..=1.0).contains(&fraction));

        let node = SplitNode {
            rect: Rect::NOTHING,
            fraction,
            fully_collapsed: self.is_collapsed(),
            collapsed_leaf_count: self.collapsed_leaf_count(),
        };

        let node = match split {
            Split::Left | Split::Right => Self::Horizontal(node),
            Split::Above | Split::Below => Self::Vertical(node),
        };

        std::mem::replace(self, node)
    }

    /// Provides an immutable slice of the tabs inside this node.
    ///
    /// Returns [`None`] if the node is not a [`Leaf`](Node::Leaf).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec![1, 2, 3, 4, 5, 6]);
    /// assert!(dock_state.main_surface().root_node().unwrap().tabs().unwrap().contains(&4));
    /// ```
    #[inline]
    pub fn get_tabs(&self) -> Option<&[Tab]> {
        match self {
            Self::Leaf(leaf) => Some(&leaf.tabs),
            _ => None,
        }
    }

    /// Provides an mutable slice of the tabs inside this node.
    ///
    /// Returns [`None`] if the node is not a [`Leaf`](Node::Leaf).
    ///
    /// # Examples
    ///
    /// Modifying tabs inside a node:
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec![1, 2, 3, 4, 5, 6]);
    /// let mut tabs = dock_state
    ///     .main_surface_mut()
    ///     .root_node_mut()
    ///     .unwrap()
    ///     .tabs_mut()
    ///     .unwrap();
    ///
    /// tabs[0] = 7;
    /// tabs[5] = 8;
    ///
    /// assert_eq!(&tabs, &[7, 2, 3, 4, 5, 8]);
    /// ```
    #[inline]
    pub fn get_tabs_mut(&mut self) -> Option<&mut [Tab]> {
        match self {
            Self::Leaf(leaf) => Some(&mut leaf.tabs),
            _ => None,
        }
    }

    /// Returns an [`Iterator`] of tabs in this node.
    ///
    /// If this node is not a [`Leaf`](Self::Leaf), then the returned [`Iterator`] will be empty.
    #[inline]
    pub fn tabs(&self) -> core::slice::Iter<'_, Tab> {
        match self {
            Self::Leaf(leaf) => leaf.tabs.iter(),
            _ => core::slice::Iter::default(),
        }
    }

    /// Returns a mutable [`Iterator`] of tabs in this node.
    ///
    /// If this node is not a [`Leaf`](Self::Leaf), then the returned [`Iterator`] will be empty.
    #[inline]
    pub fn tabs_mut(&mut self) -> core::slice::IterMut<'_, Tab> {
        match self {
            Self::Leaf(leaf) => leaf.tabs.iter_mut(),
            _ => core::slice::IterMut::default(),
        }
    }

    /// Adds `tab` to the node and sets it as the active tab.
    ///
    /// # Panics
    ///
    /// If the new capacity of `tabs` exceeds `isize::MAX` bytes.
    ///
    /// If `self` is not a [`Leaf`](Node::Leaf) node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec!["a tab"]);
    /// assert_eq!(dock_state.main_surface().root_node().unwrap().tabs_count(), 1);
    ///
    /// dock_state.main_surface_mut().root_node_mut().unwrap().append_tab("another tab");
    /// assert_eq!(dock_state.main_surface().root_node().unwrap().tabs_count(), 2);
    /// ```
    #[track_caller]
    #[inline]
    pub fn append_tab(&mut self, tab: Tab) {
        match self {
            Self::Leaf(leaf) => {
                leaf.active = TabIndex(leaf.tabs.len());
                leaf.tabs.push(tab);
            }
            _ => panic!("node was not a leaf"),
        }
    }

    /// Sets the collapsing state of the node.
    ///
    /// # Panics
    ///
    /// Panics if `self` is an [`Empty`](Node::Empty) node.
    #[inline]
    pub fn set_collapsed(&mut self, collapsed: bool) {
        match self {
            Self::Leaf(leaf) => leaf.collapsed = collapsed,
            Self::Vertical(split) | Self::Horizontal(split) => split.fully_collapsed = collapsed,
            Self::Empty => panic!("node was empty"),
        }
    }

    /// Sets the number of layers of collapsed leaf subnodes.
    ///
    /// # Panics
    ///
    /// Panics if `self` is neither a [`Vertical`](Node::Vertical) nor a [`Horizontal`](Node::Horizontal) node.
    #[inline]
    pub fn set_collapsed_leaf_count(&mut self, count: i32) {
        match self {
            Self::Horizontal(split) | Self::Vertical(split) => split.collapsed_leaf_count = count,
            _ => panic!("node was neither vertical nor horizontal"),
        }
    }

    /// Adds a `tab` to the node.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity of `tabs` exceeds `isize::MAX` bytes, or `index > tabs_count()`.
    #[track_caller]
    #[inline]
    pub fn insert_tab(&mut self, index: TabIndex, tab: Tab) {
        match self {
            Self::Leaf(leaf) => {
                leaf.tabs.insert(index.0, tab);
                leaf.active = index;
            }
            _ => panic!("node was not a leaf!"),
        }
    }

    /// Removes a tab at given `index` from the node.
    /// Returns the removed tab if the node is a `Leaf`, or `None` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove_tab(&mut self, index: TabIndex) -> Option<Tab> {
        match self {
            Self::Leaf(leaf) => Some({
                if index <= leaf.active {
                    leaf.active.0 = leaf.active.0.saturating_sub(1);
                }
                leaf.tabs.remove(index.0)
            }),
            _ => None,
        }
    }

    /// Gets the number of tabs in the node.
    #[inline]
    pub fn num_tabs(&self) -> usize {
        match self {
            Self::Leaf(leaf) => leaf.tabs.len(),
            _ => 0,
        }
    }
}
