//! Binary tree representing the relationships between [`Node`]s.
//!
//! # Implementation details
//!
//! The binary tree is stored in a [`Vec`] indexed by [`NodeIndex`].
//! The root is always at index *0*.
//! For a given node *n*:
//!  - left child of *n* will be at index *n * 2 + 1*.
//!  - right child of *n* will be at index *n * 2 + 2*.

/// Represents an abstract node of a [`Tree`].
mod node;

/// Represents an area in which a dock tree is rendered.
mod surface;

/// Wrapper around indices to the collection of nodes inside a [`Tree`].
/// Identifies a tab within a [`Node`].
/// Wrapper around indices to the collection of surfaces inside a [`DockState`].
mod index;

pub use self::index::{NodeIndex, SurfaceIndex, TabIndex};
pub use self::node::{LeafNode, Node, Split, SplitNode};
pub use self::surface::{Surface, WindowState};

use egui::Rect;
use std::{fmt, ops, slice};

/// An enum expressing an entry in the `to_remove` field in [`DockArea`].
#[derive(Debug, Clone, Copy)]
pub(crate) enum TabRemoval {
    Tab(SurfaceIndex, NodeIndex, TabIndex),
    ForcedTab(SurfaceIndex, NodeIndex, TabIndex),
    Node(SurfaceIndex, NodeIndex),
    Window(SurfaceIndex),
}

/// The heart of `egui_dock`.
///
/// This structure holds a collection of surfaces, each of which stores a tree in which tabs are arranged.
///
/// Indexing it with a [`SurfaceIndex`] will yield a [`Tree`] which then contains nodes and tabs.
///
/// [`DockState`] is generic, so you can use any type of data to represent a tab.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DockState<Tab> {
    pub(crate) surfaces: Vec<Surface<Tab>>,
    pub(crate) focused: Option<SurfaceIndex>, // Part of the tree which is in focus.
}

impl<Tab> std::ops::Index<SurfaceIndex> for DockState<Tab> {
    type Output = Tree<Tab>;

    #[inline(always)]
    fn index(&self, index: SurfaceIndex) -> &Self::Output {
        match &self.surfaces[index.0] {
            Surface::Empty => panic!("There did not exist a tree at surface index {index:?}"),
            Surface::Main(tree) | Surface::Window(tree, _) => tree,
        }
    }
}

impl<Tab> std::ops::IndexMut<SurfaceIndex> for DockState<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: SurfaceIndex) -> &mut Self::Output {
        match &mut self.surfaces[index.0] {
            Surface::Empty => panic!("There did not exist a tree at surface index {index:?}"),
            Surface::Main(tree) | Surface::Window(tree, _) => tree,
        }
    }
}

impl<Tab> DockState<Tab> {
    /// Create a new tree with given tabs at the main surface's root node.
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            surfaces: vec![Surface::Main(Tree::new(tabs))],
            focused: None,
        }
    }

    pub(crate) fn set_rect(&mut self, surface: SurfaceIndex, node: NodeIndex, rect: Rect) {
        match &mut self.surfaces[surface.0] {
            Surface::Empty => panic!("There did not exist a tree at {surface:?}"),
            Surface::Main(tree) | Surface::Window(tree, _) => match &mut tree[node] {
                Node::Empty => (),
                Node::Leaf(leaf) => leaf.rect = rect,
                Node::Vertical(split) | Node::Horizontal(split) => split.rect = rect,
            },
        }
    }

    /// Get an immutable borrow to the tree at the main surface.
    pub fn main_surface(&self) -> &Tree<Tab> {
        &self[SurfaceIndex::main()]
    }

    /// Get a mutable borrow to the tree at the main surface.
    pub fn main_surface_mut(&mut self) -> &mut Tree<Tab> {
        &mut self[SurfaceIndex::main()]
    }

    /// Get the [`WindowState`] which corresponds to a [`SurfaceIndex`].
    pub fn window(&self, surface: SurfaceIndex) -> Option<(&Tree<Tab>, &WindowState)> {
        match &self.surfaces[surface.0] {
            Surface::Window(tree, state) => Some((tree, state)),
            _ => None,
        }
    }

    /// Get the [`WindowState`] which corresponds to a [`SurfaceIndex`].
    ///
    /// Returns `None` if the surface is [`Empty`](Surface::Empty), [`Main`](Surface::Main), or doesn't exist.
    ///
    /// This can be used to modify properties of a window, e.g. size and position.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::DockState;
    /// # use egui::{Vec2, Pos2};
    /// let mut dock_state = DockState::new(vec![]);
    /// let mut surface_index = dock_state.add_window(vec!["Window Tab".to_string()]);
    /// let window_state = dock_state.get_window_state_mut(surface_index).unwrap();
    ///
    /// window_state.set_position(Pos2::ZERO);
    /// window_state.set_size(Vec2::splat(100.0));
    /// ```
    pub fn window_mut(
        &mut self,
        surface: SurfaceIndex,
    ) -> Option<(&mut Tree<Tab>, &mut WindowState)> {
        match &mut self.surfaces[surface.0] {
            Surface::Window(tree, state) => Some((tree, state)),
            _ => None,
        }
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the focused leaf node or `None` if no node is in focus.
    #[inline]
    pub fn active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        self.focused.and_then(|surface| {
            let surface = &mut self[surface];
            let node = surface.focused.and_then(|i| surface.nodes.get_mut(i.0));
            if let Some(Node::Leaf(leaf)) = node {
                let TabIndex(active) = leaf.active;
                leaf.tabs.get_mut(active).map(|tab| (leaf.viewport, tab))
            } else {
                None
            }
        })
    }

    /// Get a mutable borrow to the raw surface from a surface index.
    #[inline]
    pub fn get_surface_mut(&mut self, surface: SurfaceIndex) -> Option<&mut Surface<Tab>> {
        self.surfaces.get_mut(surface.0)
    }

    /// Get an immutable borrow to the raw surface from a surface index.
    #[inline]
    pub fn get_surface(&self, surface: SurfaceIndex) -> Option<&Surface<Tab>> {
        self.surfaces.get(surface.0)
    }

    /// Returns true if the specified surface exists and isn't [`Empty`](Surface::Empty).
    #[inline]
    pub fn is_surface_valid(&self, surface_index: SurfaceIndex) -> bool {
        self.surfaces
            .get(surface_index.0)
            .is_some_and(|surface| !matches!(surface, Surface::Empty))
    }

    /// Returns a list of all valid [`SurfaceIndex`]es.
    #[inline]
    pub(crate) fn valid_surface_indices_iter(&self) -> impl Iterator<Item = SurfaceIndex> {
        (0..self.surfaces.len())
            .filter(|&index| !matches!(self.surfaces[index], Surface::Empty))
            .map(SurfaceIndex)
    }

    /// Remove a surface based on its [`SurfaceIndex`]
    ///
    /// Returns the removed surface or `None` if it didn't exist.
    ///
    /// # Panics
    ///
    /// Panics if you try to remove the main surface: `SurfaceIndex::main()`.
    pub fn remove_surface(&mut self, surface_index: SurfaceIndex) -> Option<Surface<Tab>> {
        assert!(!surface_index.is_main());
        (surface_index.0 < self.surfaces.len()).then(|| {
            self.focused = Some(SurfaceIndex::main());
            if surface_index.0 == self.surfaces.len() - 1 {
                self.surfaces.pop().unwrap()
            } else {
                let dest = &mut self.surfaces[surface_index.0];
                std::mem::replace(dest, Surface::Empty)
            }
        })
    }

    /// Sets which is the active tab within a specific node on a given surface.
    #[inline]
    pub fn set_active_tab(
        &mut self,
        (surface_index, node_index, tab_index): (SurfaceIndex, NodeIndex, TabIndex),
    ) {
        if let Some(Node::Leaf(leaf)) = self[surface_index].nodes.get_mut(node_index.0) {
            leaf.active = tab_index;
        }
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    #[inline]
    pub fn set_focused_node_and_surface(
        &mut self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
    ) {
        if self.is_surface_valid(surface_index) && node_index.0 < self[surface_index].len() {
            // I don't want this code to be evaluated until im absolutely sure the surface index is valid.
            if self[surface_index][node_index].is_leaf() {
                self.focused = Some(surface_index);
                self[surface_index].set_focused_node(node_index);
                return;
            }
        }
        self.focused = None;
    }

    /// Moves a tab from a node to another node.
    /// You need to specify with [`TabDestination`] how the tab should be moved.
    pub fn move_tab(
        &mut self,
        (src_surface, src_node, src_tab): (SurfaceIndex, NodeIndex, TabIndex),
        dst_tab: impl Into<TabDestination>,
    ) {
        match dst_tab.into() {
            TabDestination::Window(position) => {
                self.detach_tab((src_surface, src_node, src_tab), position);
                return;
            }
            TabDestination::Node(dst_surface, dst_node, dst_tab) => {
                // Moving a single tab inside its own node is a no-op
                if src_surface == dst_surface
                    && src_node == dst_node
                    && self[src_surface][src_node].num_tabs() == 1
                {
                    return;
                }

                // Call `Node::remove_tab` to avoid auto remove of the node by `Tree::remove_tab` from Tree.
                let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();
                match dst_tab {
                    TabInsert::Split(split) => {
                        self[dst_surface].split(dst_node, split, 0.5, Node::new(vec![tab]));
                    }

                    TabInsert::Insert(index) => self[dst_surface][dst_node].insert_tab(index, tab),
                    TabInsert::Append => self[dst_surface][dst_node].append_tab(tab),
                }
            }
            TabDestination::EmptySurface(dst_surface) => {
                assert!(self[dst_surface].is_empty());
                let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();
                self[dst_surface] = Tree::new(vec![tab]);
            }
        }
        if self[src_surface][src_node].is_leaf() && self[src_surface][src_node].num_tabs() == 0 {
            self[src_surface].remove_leaf(src_node);
        }
        if self[src_surface].is_empty() && !src_surface.is_main() {
            self.remove_surface(src_surface);
        }
    }

    /// Takes a tab out of its current surface and puts it in a new window.
    /// Returns the surface index of the new window.
    pub fn detach_tab(
        &mut self,
        (src_surface, src_node, src_tab): (SurfaceIndex, NodeIndex, TabIndex),
        window_rect: Rect,
    ) -> SurfaceIndex {
        // Remove the tab from the tree and it add to a new window.
        let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();
        let surface_index = self.add_window(vec![tab]);

        // Set the window size and position to match `window_rect`.
        let (_, state) = self.window_mut(surface_index).unwrap();
        state.set_position(window_rect.min);
        if src_surface.is_main() {
            state.set_size(window_rect.size() * 0.8);
        } else {
            state.set_size(window_rect.size());
        }

        // Clean up any empty leaves and surfaces which may be left behind from the detachment.
        if self[src_surface][src_node].is_leaf() && self[src_surface][src_node].num_tabs() == 0 {
            self[src_surface].remove_leaf(src_node);
        }
        if self[src_surface].is_empty() && !src_surface.is_main() {
            self.remove_surface(src_surface);
        }
        surface_index
    }

    /// Currently focused leaf.
    #[inline]
    pub fn focused_leaf(&self) -> Option<(SurfaceIndex, NodeIndex)> {
        let surface = self.focused?;
        self[surface].focused_leaf().map(|leaf| (surface, leaf))
    }

    /// Remove a tab at the specified surface, node, and tab index.
    /// This method will yield the removed tab, or `None` if it doesn't exist.
    pub fn remove_tab(
        &mut self,
        surface: SurfaceIndex,
        node: NodeIndex,
        tab: TabIndex,
    ) -> Option<Tab> {
        let removed_tab = self[surface].remove_tab(node, tab);
        if !surface.is_main() && self[surface].is_empty() {
            self.remove_surface(surface);
        }
        removed_tab
    }

    /// Remove a leaf at the specified surface, and node index.
    pub fn remove_leaf(&mut self, (surface_index, node_index): (SurfaceIndex, NodeIndex)) {
        self[surface_index].remove_leaf(node_index);
        if !surface_index.is_main() && self[surface_index].is_empty() {
            self.remove_surface(surface_index);
        }
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split(
        &mut self,
        (surface, parent): (SurfaceIndex, NodeIndex),
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        self.focused = Some(surface);
        self[surface].split(parent, split, fraction, new)
    }

    /// Adds a window with its own list of tabs.
    ///
    /// Returns the [`SurfaceIndex`] of the new window, which will remain constant through the windows lifetime.
    pub fn add_window(&mut self, tabs: Vec<Tab>) -> SurfaceIndex {
        let surface = Surface::Window(Tree::new(tabs), WindowState::default());

        // Find the first possible empty surface to insert our window into.
        // Starts at 1 as 0 is always the main surface.
        if let Some(index) =
            (1..self.surfaces.len()).find(|&i| matches!(self.surfaces[i], Surface::Empty))
        {
            self.surfaces[index] = surface;
            SurfaceIndex(index)
        } else {
            self.surfaces.push(surface);
            SurfaceIndex(self.surfaces.len() - 1)
        }
    }

    /// Ensures that the surface at `index` contains a [`Tree`]
    ///
    /// If the surface is [`Empty`](Surface::Empty), builds a [`Surface::Main`]
    /// for the main surface or a [`Surface::Window`] for other surfaces.
    ///
    /// # Panics
    /// If `index` is not a valid `SurfaceIndex`
    fn ensure_tree(&mut self, index: SurfaceIndex) {
        if let Surface::Empty = self.surfaces[index.0] {
            self.surfaces[index.0] = if index.is_main() {
                Surface::Main(Tree::new(vec![]))
            } else {
                Surface::Window(Tree::new(vec![]), WindowState::default())
            }
        }
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        let surface_index = self.focused.unwrap_or(SurfaceIndex::main());
        self.ensure_tree(surface_index);
        self[surface_index].push_to_focused_leaf(tab);
    }

    /// Push a tab to the first available `Leaf` or create a new leaf if an `Empty` node is encountered.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        self.ensure_tree(SurfaceIndex::main());
        self[SurfaceIndex::main()].push_to_first_leaf(tab);
    }
}

// ----------------------------------------------------------------------------

/// Specify how a tab should be added to a Node.
pub enum TabInsert {
    /// Split the node in the given direction.
    Split(Split),

    /// Insert the tab at the given index.
    Insert(TabIndex),

    /// Append the tab to the node.
    Append,
}

/// The destination for a tab which is being moved.
pub enum TabDestination {
    /// Move to a new window with this rect.
    Window(Rect),

    /// Move to a an existing node with this insertion.
    Node(SurfaceIndex, NodeIndex, TabInsert),

    /// Move to an empty surface.
    EmptySurface(SurfaceIndex),
}

impl From<(SurfaceIndex, NodeIndex, TabInsert)> for TabDestination {
    fn from(value: (SurfaceIndex, NodeIndex, TabInsert)) -> TabDestination {
        Self::Node(value.0, value.1, value.2)
    }
}

impl From<SurfaceIndex> for TabDestination {
    fn from(value: SurfaceIndex) -> TabDestination {
        Self::EmptySurface(value)
    }
}

impl TabDestination {
    /// Returns if this tab destination is a [`Window`](TabDestination::Window).
    pub fn is_window(&self) -> bool {
        matches!(self, Self::Window(_))
    }
}

/// Binary tree representing the relationships between [`Node`]s.
///
/// # Implementation details
///
/// The binary tree is stored in a [`Vec`] indexed by [`NodeIndex`].
/// The root is always at index *0*.
/// For a given node *n*:
///  - left child of *n* will be at index *n * 2 + 1*.
///  - right child of *n* will be at index *n * 2 + 2*.
///
/// For "Horizontal" nodes:
///  - left child contains Left node.
///  - right child contains Right node.
///
/// For "Vertical" nodes:
///  - left child contains Top node.
///  - right child contains Bottom node.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Tree<Tab> {
    // Binary tree vector
    pub(crate) nodes: Vec<Node<Tab>>,
    pub(crate) focused: Option<NodeIndex>,
    // Whether all subnodes of the tree is collapsed
    pub(crate) collapsed: bool,
    pub(crate) collapsed_leaf_count: i32,
}

impl<Tab> fmt::Debug for Tree<Tab> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree").finish_non_exhaustive()
    }
}

impl<Tab> Default for Tree<Tab> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            focused: None,
            collapsed: false,
            collapsed_leaf_count: 0,
        }
    }
}

impl<Tab> ops::Index<NodeIndex> for Tree<Tab> {
    type Output = Node<Tab>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<Tab> ops::IndexMut<NodeIndex> for Tree<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<Tab> Tree<Tab> {
    /// Creates a new [`Tree`] with given `Vec` of `Tab`s in its root node.
    #[inline(always)]
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            nodes: vec![Node::new(tabs)],
            focused: None,
            collapsed: false,
            collapsed_leaf_count: 0,
        }
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the first leaf node,
    /// or `None` if no leaf exists in the [`Tree`].
    #[inline]
    pub fn find_active(&mut self) -> Option<(Rect, &mut Tab)> {
        self.nodes.iter_mut().find_map(|node| match node {
            Node::Leaf(leaf) => leaf
                .tabs
                .get_mut(leaf.active.0)
                .map(|tab| (leaf.viewport, tab)),
            _ => None,
        })
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the focused leaf node or [`None`] if it does not exist.
    #[inline]
    pub fn find_active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        match self.focused.and_then(|i| self.nodes.get_mut(i.0)) {
            Some(Node::Leaf(leaf)) => leaf
                .tabs
                .get_mut(leaf.active.0)
                .map(|tab| (leaf.viewport, tab)),
            _ => None,
        }
    }

    /// Returns the number of nodes in the [`Tree`].
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the number of nodes in the tree is 0, otherwise `false`.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an [`Iterator`] of the underlying collection of nodes.
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<'_, Node<Tab>> {
        self.nodes.iter()
    }

    /// Returns [`IterMut`] of the underlying collection of nodes.
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Node<Tab>> {
        self.nodes.iter_mut()
    }

    /// Returns an [`Iterator`] of [`NodeIndex`] ordered in a breadth first manner.
    #[inline(always)]
    pub(crate) fn breadth_first_index_iter(&self) -> impl Iterator<Item = NodeIndex> + use<Tab> {
        (0..self.nodes.len()).map(NodeIndex)
    }

    /// Returns an iterator over all tabs in breadth-first order.
    #[inline(always)]
    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.nodes.iter().flat_map(Node::tabs)
    }

    #[inline]
    pub fn leafs(&self) -> impl Iterator<Item = &LeafNode<Tab>> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Leaf(leaf) => Some(leaf),
            _ => None,
        })
    }

    /// Acquire a immutable borrow to the [`Node`] at the root of the tree.
    /// Returns [`None`] if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::DockState;
    /// let mut dock_state = DockState::new(vec!["single tab"]);
    /// let root_node = dock_state.main_surface().root_node().unwrap();
    ///
    /// assert_eq!(root_node.tabs(), Some(["single tab"].as_slice()));
    /// ```
    pub fn root_node(&self) -> Option<&Node<Tab>> {
        self.nodes.first()
    }

    /// Acquire a mutable borrow to the [`Node`] at the root of the tree.
    /// Returns [`None`] if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, LeafNode};
    /// let mut dock_state = DockState::new(vec!["single tab"]);
    /// let root_node = dock_state.main_surface_mut().root_node_mut().unwrap();
    /// let root_as_leaf = root_node.get_leaf_mut().unwrap();
    /// root_as_leaf.tabs.push("partner tab");
    ///
    /// assert_eq!(root_node.tabs(), Some(["single tab", "partner tab"].as_slice()));
    /// ```
    pub fn root_node_mut(&mut self) -> Option<&mut Node<Tab>> {
        self.nodes.first_mut()
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_dock::{DockState, SurfaceIndex, NodeIndex, Split};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    ///
    /// // At this point, the main surface only contains the leaf with tab 1 and 2.
    /// assert!(dock_state.main_surface().root_node().unwrap().is_leaf());
    ///
    /// // Split the node, giving 50% of the space to the new nodes and 50% to the old ones.
    /// let [old, new] = dock_state.main_surface_mut()
    ///     .split_tabs(NodeIndex::root(), Split::Below, 0.5, vec!["tab 3"]);
    ///
    /// assert!(dock_state.main_surface().root_node().unwrap().is_parent());
    /// assert!(dock_state[SurfaceIndex::main()][old].is_leaf());
    /// assert!(dock_state[SurfaceIndex::main()][new].is_leaf());
    /// ```
    #[inline(always)]
    pub fn split_tabs(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, split, fraction, Node::new(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Above`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed *above* the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_above(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Above, fraction, Node::new(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Below`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed *below* the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_below(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Below, fraction, Node::new(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Left`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed to the *left* of the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_left(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Left, fraction, Node::new(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Right`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed to the *right* of the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_right(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Right, fraction, Node::new(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) uses `new`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    ///
    /// If `new` is an [`Empty`](Node::Empty), [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical) node.
    ///
    /// If `new` is a [`Leaf`](Node::Leaf) node without any tabs.
    ///
    /// If `parent` points to an [`Empty`](Node::Empty) node.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_dock::{DockState, SurfaceIndex, NodeIndex, Split, Node};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    ///
    /// // At this point, the main surface only contains the leaf with tab 1 and 2.
    /// assert!(dock_state.main_surface().root_node().unwrap().is_leaf());
    ///
    /// // Splits the node, giving 50% of the space to the new nodes and 50% to the old ones.
    /// let [old, new] = dock_state.main_surface_mut()
    ///     .split(NodeIndex::root(), Split::Below, 0.5, Node::leaf_with(vec!["tab 3"]));
    ///
    /// assert!(dock_state.main_surface().root_node().unwrap().is_parent());
    /// assert!(dock_state[SurfaceIndex::main()][old].is_leaf());
    /// assert!(dock_state[SurfaceIndex::main()][new].is_leaf());
    /// ```
    pub fn split(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        let old = self[parent].split(split, fraction);
        assert!(old.is_leaf() || old.is_parent());
        assert_ne!(new.num_tabs(), 0);
        // Resize vector to fit the new size of the binary tree.
        {
            let index = self.nodes.iter().rposition(|n| !n.is_empty()).unwrap_or(0);
            let level = NodeIndex(index).level();
            self.nodes
                .resize_with((1 << (level + 1)) - 1, || Node::Empty);
        }

        let index = match split {
            Split::Left | Split::Above => [parent.right(), parent.left()],
            Split::Right | Split::Below => [parent.left(), parent.right()],
        };

        // If the node were splitting is a parent, all it's children need to be moved.
        if old.is_parent() {
            let levels_to_move = NodeIndex(self.nodes.len()).level() - index[0].level();

            // Level 0 is ourself, which is done when we assign self[index[0]] = old, so start at 1.
            for level in (1..levels_to_move).rev() {
                // Old child indices for this level
                let old_start = parent.children_at(level).start;
                // New child indices for this level
                let new_start = index[0].children_at(level).start;

                // Children to be moved this level change
                let len = 1 << level;

                // Swap self[old_start..(old_start+len)] with self[new_start..(new_start+len)]
                // (the new part will only contain empty entries).
                let (old_range, new_range) = {
                    let (first_part, second_part) = self.nodes.split_at_mut(new_start);
                    // Cut to length.
                    (
                        &mut first_part[old_start..old_start + len],
                        &mut second_part[..len],
                    )
                };
                old_range.swap_with_slice(new_range);
            }
        }

        self[index[0]] = old;
        self[index[1]] = new;

        self.focused = Some(index[1]);
        self.node_update_collapsed(index[1]);

        index
    }

    fn first_leaf(&self, top: NodeIndex) -> Option<NodeIndex> {
        let left = top.left();
        let right = top.right();
        match (self.nodes.get(left.0), self.nodes.get(right.0)) {
            (Some(Node::Leaf { .. }), _) => Some(left),
            (_, Some(Node::Leaf { .. })) => Some(right),

            (
                Some(Node::Horizontal { .. } | Node::Vertical { .. }),
                Some(Node::Horizontal { .. } | Node::Vertical { .. }),
            ) => self.first_leaf(left).or(self.first_leaf(right)),
            (Some(Node::Horizontal { .. } | Node::Vertical { .. }), _) => self.first_leaf(left),
            (_, Some(Node::Horizontal { .. } | Node::Vertical { .. })) => self.first_leaf(right),

            (None | Some(Node::Empty), None | Some(Node::Empty)) => None,
        }
    }

    /// Gets the node index of currently focused leaf node; returns [`None`] when no leaf is focused.
    #[inline]
    pub fn focused_leaf(&self) -> Option<NodeIndex> {
        self.focused
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    ///
    /// This method will not never panic and instead removes focus from all nodes when given an invalid index.
    #[inline]
    pub fn set_focused_node(&mut self, node_index: NodeIndex) {
        self.focused = self
            .nodes
            .get(node_index.0)
            .filter(|node| node.is_leaf())
            .map(|_| node_index);
    }

    /// Removes the given node from the [`Tree`].
    ///
    /// # Panics
    ///
    /// - If the tree is empty.
    /// - If the node at index `node` is not a [`Leaf`](Node::Leaf).
    pub fn remove_leaf(&mut self, node: NodeIndex) {
        assert!(!self.is_empty());
        assert!(self[node].is_leaf());

        let Some(parent) = node.parent() else {
            self.nodes.clear();
            return;
        };

        if Some(node) == self.focused {
            self.focused = None;
            let mut node = node;
            while let Some(parent) = node.parent() {
                let next = if node.is_left() {
                    parent.right()
                } else {
                    parent.left()
                };
                if self.nodes.get(next.0).is_some_and(Node::is_leaf) {
                    self.focused = Some(next);
                    break;
                }
                if let Some(node) = self.first_leaf(next) {
                    self.focused = Some(node);
                    break;
                }
                node = parent;
            }
        }

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
                    if Some(NodeIndex(src)) == self.focused {
                        self.focused = Some(NodeIndex(dst));
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
                    if Some(NodeIndex(src)) == self.focused {
                        self.focused = Some(NodeIndex(dst));
                    }
                    self.nodes[dst] = std::mem::replace(&mut self.nodes[src], Node::Empty);
                }
                level += 1;
            }
        }
        // Ensure that there are no trailing `Node::Empty` items
        while let Some(last_index) = self.nodes.len().checked_sub(1).map(NodeIndex) {
            if self[last_index].is_empty()
                && last_index.parent().is_some_and(|pi| !self[pi].is_parent())
            {
                self.nodes.pop();
            } else {
                break;
            }
        }
    }

    /// Pushes a tab to the first `Leaf` it finds or create a new leaf if an `Empty` node is encountered.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        for (index, node) in &mut self.nodes.iter_mut().enumerate() {
            match node {
                Node::Leaf(leaf) => {
                    leaf.active = TabIndex(leaf.tabs.len());
                    leaf.tabs.push(tab);
                    self.focused = Some(NodeIndex(index));
                    return;
                }
                Node::Empty => {
                    *node = Node::new(vec![tab]);
                    self.focused = Some(NodeIndex(index));
                    return;
                }
                _ => {}
            }
        }
        assert!(self.nodes.is_empty());
        self.nodes.push(Node::new(vec![tab]));
        self.focused = Some(NodeIndex(0));
    }

    /// Sets which is the active tab within a specific node.
    #[inline]
    pub fn set_active_tab(
        &mut self,
        node_index: impl Into<NodeIndex>,
        tab_index: impl Into<TabIndex>,
    ) {
        if let Some(Node::Leaf(leaf)) = self.nodes.get_mut(node_index.into().0) {
            let index = tab_index.into();
            if index.0 < leaf.tabs.len() {
                leaf.active = index;
            }
        }
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        match self.focused {
            _ if self.nodes.is_empty() => {
                self.nodes.push(Node::new(vec![tab]));
                self.focused = Some(NodeIndex::root());
            }
            Some(node) => match &mut self[node] {
                Node::Empty => {
                    self[node] = Node::new(vec![tab]);
                    self.focused = Some(node);
                }
                Node::Leaf(leaf) => {
                    leaf.active = TabIndex(leaf.tabs.len());
                    leaf.tabs.push(tab);
                    self.focused = Some(node);
                }
                _ => self.push_to_first_leaf(tab),
            },
            None => self.push_to_first_leaf(tab),
        }
    }

    /// Removes the tab at the given ([`NodeIndex`], [`TabIndex`]) pair.
    ///
    /// If the node is emptied after the tab is removed, the node will also be removed.
    ///
    /// Returns the removed tab if it exists, or `None` otherwise.
    pub fn remove_tab(&mut self, node_index: NodeIndex, tab_index: TabIndex) -> Option<Tab> {
        let node = &mut self[node_index];
        let tab = node.remove_tab(tab_index);
        if node.num_tabs() == 0 {
            self.remove_leaf(node_index);
        }
        tab
    }

    /// Sets the collapsing state of the [`Tree`].
    pub(crate) fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    /// Returns whether the [`Tree`] is collapsed.
    pub(crate) fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Sets the number of collapsed layers of leaf subnodes in the [`Tree`].
    pub(crate) fn set_collapsed_leaf_count(&mut self, collapsed_leaf_count: i32) {
        self.collapsed_leaf_count = collapsed_leaf_count;
    }

    /// Returns the number of collapsed layers of leaf subnodes in the [`Tree`].
    pub(crate) fn collapsed_leaf_count(&self) -> i32 {
        self.collapsed_leaf_count
    }

    /// Updates the collapsed state of the node and its parents.
    pub(crate) fn node_update_collapsed(&mut self, node_index: NodeIndex) {
        if self[node_index].is_collapsed() {
            // Recursively notify parent nodes that the leaf has collapsed
            let mut parent_index_option = node_index.parent();
            while let Some(parent_index) = parent_index_option {
                parent_index_option = parent_index.parent();

                // Update collapsed leaf count and collapse status
                let left_count = self[parent_index.left()].collapsed_leaf_count();
                let right_count = self[parent_index.right()].collapsed_leaf_count();

                if self[parent_index].is_horizontal() {
                    self[parent_index].set_collapsed_leaf_count(i32::max(left_count, right_count));
                } else {
                    self[parent_index].set_collapsed_leaf_count(left_count + right_count);
                }

                if self[parent_index.left()].is_collapsed()
                    && self[parent_index.right()].is_collapsed()
                {
                    self[parent_index].set_collapsed(true);
                }
            }
            if self.root_node().is_some_and(Node::is_collapsed) {
                self.set_collapsed(true);
                self.set_collapsed_leaf_count(self[NodeIndex::root()].collapsed_leaf_count());
            }
        } else {
            // Recursively notify parent nodes that the leaf has expanded
            let mut parent_index_option = node_index.parent();
            while let Some(parent_index) = parent_index_option {
                parent_index_option = parent_index.parent();

                // Update collapsed leaf count and collapse status
                let left_count = self[parent_index.left()].collapsed_leaf_count();
                let right_count = self[parent_index.right()].collapsed_leaf_count();
                self[parent_index].set_collapsed(false);

                if self[parent_index].is_horizontal() {
                    self[parent_index].set_collapsed_leaf_count(i32::max(left_count, right_count));
                } else {
                    self[parent_index].set_collapsed_leaf_count(left_count + right_count);
                }
            }
            self.set_collapsed(false);
            self.set_collapsed_leaf_count(self[NodeIndex::root()].collapsed_leaf_count());
        }
    }
}
