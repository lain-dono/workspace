use super::dock_state::DockState;
use super::tree::{LeafNode, Node, NodeIndex, Surface, SurfaceIndex, TabIndex, Tree};
use bevy_egui::egui::ahash::HashSet;

impl<Tab> LeafNode<Tab> {
    /// Get the length of tab list in this [`LeafNode`].
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns `true` when the [`LeafNode`] contains no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Removes all tabs for which `predicate` returns `false`.
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        self.tabs.retain_mut(predicate);
    }
}

impl<Tab> Node<Tab> {
    /// Returns a new [`Node`] while mapping and filtering the tab type.
    /// If this [`Node`] remains empty, it will change to [`Node::Empty`].
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> Node<NewTab>
    where
        F: FnMut(&Tab) -> Option<NewTab>,
    {
        match self {
            Self::Leaf(leaf) => {
                let tabs: Vec<_> = leaf.tabs.iter().filter_map(function).collect();
                if tabs.is_empty() {
                    Node::Empty
                } else {
                    Node::Leaf(LeafNode {
                        rect: leaf.rect,
                        viewport: leaf.viewport,
                        tabs,
                        active: leaf.active,
                        scroll: leaf.scroll,
                        collapsed: leaf.collapsed,
                    })
                }
            }
            Self::Empty => Node::Empty,
            Self::Vertical(split) => Node::Vertical(split.clone()),
            Self::Horizontal(split) => Node::Horizontal(split.clone()),
        }
    }

    /// Returns a new [`Node`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Node<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Node`] while filtering the tab type.
    /// If this [`Node`] remains empty, it will change to [`Node::Empty`].
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Node<Tab>
    where
        F: FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// If this [`Node`] remains empty, it will change to [`Node::Empty`].
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        if let Node::Leaf(leaf) = self {
            leaf.retain_tabs(predicate);
            if leaf.tabs.is_empty() {
                *self = Self::Empty;
            }
        }
    }
}

impl<Tab> Surface<Tab> {
    /// Returns an [`Iterator`] of nodes in this surface's tree.
    ///
    /// If the surface is [`Empty`](Self::Empty), then the returned [`Iterator`] will be empty.
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
        match self {
            Self::Empty => core::slice::Iter::default(),
            Self::Main(tree) | Self::Window(tree, _) => tree.iter(),
        }
    }

    /// Returns a mutable [`Iterator`] of nodes in this surface's tree.
    ///
    /// If the surface is [`Empty`](Self::Empty), then the returned [`Iterator`] will be empty.
    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<Tab>> {
        match self {
            Self::Empty => core::slice::IterMut::default(),
            Self::Main(tree) | Self::Window(tree, _) => tree.iter_mut(),
        }
    }

    /// Returns an [`Iterator`] of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs(&self) -> impl Iterator<Item = (NodeIndex, &Tab)> {
        self.iter_nodes()
            .enumerate()
            .flat_map(|(index, node)| node.iter_tabs().map(move |tab| (NodeIndex(index), tab)))
    }

    /// Returns a mutable [`Iterator`] of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs_mut(&mut self) -> impl Iterator<Item = (NodeIndex, &mut Tab)> {
        self.iter_nodes_mut()
            .enumerate()
            .flat_map(|(index, node)| node.iter_tabs_mut().map(move |tab| (NodeIndex(index), tab)))
    }

    /// Returns a new [`Surface`] while mapping and filtering the tab type.
    /// Any remaining empty [`Node`]s and are removed, and if this [`Surface`] remains empty,
    /// it'll change to [`Surface::Empty`].
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> Surface<NewTab>
    where
        F: FnMut(&Tab) -> Option<NewTab>,
    {
        match self {
            Self::Empty => Surface::Empty,
            Self::Main(tree) => Surface::Main(tree.filter_map_tabs(function)),
            Self::Window(tree, window_state) => {
                let tree = tree.filter_map_tabs(function);
                if tree.is_empty() {
                    Surface::Empty
                } else {
                    Surface::Window(tree, window_state.clone())
                }
            }
        }
    }

    /// Returns a new [`Surface`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Surface<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Surface`] while filtering the tab type.
    /// Any remaining empty [`Node`]s and are removed, and if this [`Surface`] remains empty,
    /// it'll change to [`Surface::Empty`].
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Self
    where
        F: FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// Any remaining empty [`Node`]s and are also removed, and if this [`Surface`] remains empty,
    /// it'll change to [`Surface::Empty`].
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        if let Self::Main(tree) | Self::Window(tree, _) = self {
            tree.retain_tabs(predicate);
            if tree.is_empty() {
                *self = Surface::Empty;
            }
        }
    }
}

impl<Tab: PartialEq> Tree<Tab> {
    /// Find the given tab.
    ///
    /// Returns in which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, TabIndex)> {
        self.find_tab_from(|tab| tab == needle_tab)
    }
}

impl<Tab> Tree<Tab> {
    /// Find a given tab based on ``predicate``.
    ///
    /// Returns the indices in where that node and tab is in this surface.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_tab_from(&self, predicate: impl Fn(&Tab) -> bool) -> Option<(NodeIndex, TabIndex)> {
        for (node_index, node) in self.nodes.iter().enumerate() {
            if let Some(tabs) = node.tabs() {
                for (tab_index, tab) in tabs.iter().enumerate() {
                    if predicate(tab) {
                        return Some((node_index.into(), tab_index.into()));
                    }
                }
            }
        }
        None
    }

    fn balance(&mut self, emptied_nodes: HashSet<NodeIndex>) {
        let mut emptied_parents = HashSet::default();
        for parent_index in emptied_nodes.into_iter().filter_map(NodeIndex::parent) {
            if !self[parent_index].is_parent() {
                continue;
            }

            if self[parent_index.left()].is_empty() && self[parent_index.right()].is_empty() {
                self[parent_index] = Node::Empty;
                emptied_parents.insert(parent_index);
            } else if self[parent_index.left()].is_empty() {
                self.nodes.swap(parent_index.0, parent_index.right().0);
                self[parent_index.right()] = Node::Empty;
            } else if self[parent_index.right()].is_empty() {
                self.nodes.swap(parent_index.0, parent_index.left().0);
                self[parent_index.left()] = Node::Empty;
            }
        }
        if !emptied_parents.is_empty() {
            self.balance(emptied_parents);
        }
    }

    /// Returns a new [`Tree`] while mapping and filtering the tab type.
    /// Any remaining empty [`Node`]s are removed.
    pub fn filter_map_tabs<F, NewTab>(&self, mut function: F) -> Tree<NewTab>
    where
        F: FnMut(&Tab) -> Option<NewTab>,
    {
        let mut emptied_nodes = HashSet::default();
        let nodes = self.nodes.iter().enumerate().map(|(index, node)| {
            let filtered_node = node.filter_map_tabs(&mut function);
            if filtered_node.is_empty() && !node.is_empty() {
                emptied_nodes.insert(NodeIndex(index));
            }
            filtered_node
        });

        let mut new_tree = Tree {
            nodes: nodes.collect(),
            focused_node: self.focused_node,
            collapsed: self.collapsed,
            collapsed_leaf_count: self.collapsed_leaf_count,
        };
        new_tree.balance(emptied_nodes);
        new_tree
    }

    /// Returns a new [`Tree`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Tree<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Tree`] while filtering the tab type.
    /// Any remaining empty [`Node`]s are removed.
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Tree<Tab>
    where
        F: FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// Any remaining empty [`Node`]s are also removed.
    pub fn retain_tabs<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        let mut emptied_nodes = HashSet::default();
        for (index, node) in self.nodes.iter_mut().enumerate() {
            node.retain_tabs(&mut predicate);
            if node.is_empty() {
                emptied_nodes.insert(NodeIndex(index));
            }
        }
        self.balance(emptied_nodes);
    }
}

impl<Tab> DockState<Tab> {
    /// Returns the current number of surfaces.
    pub fn surfaces_count(&self) -> usize {
        self.surfaces.len()
    }

    /// Returns an [`Iterator`] over all surfaces.
    pub fn iter_surfaces(&self) -> impl Iterator<Item = &Surface<Tab>> {
        self.surfaces.iter()
    }

    /// Returns a mutable [`Iterator`] over all surfaces.
    pub fn iter_surfaces_mut(&mut self) -> impl Iterator<Item = &mut Surface<Tab>> {
        self.surfaces.iter_mut()
    }

    /// Returns an [`Iterator`] of **all** underlying nodes in the dock state,
    /// and the indices of containing surfaces.
    pub fn iter_all_nodes(&self) -> impl Iterator<Item = (SurfaceIndex, &Node<Tab>)> {
        self.iter_surfaces()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_nodes()
                    .map(move |node| (surface_index.into(), node))
            })
    }

    /// Returns a mutable [`Iterator`] of **all** underlying nodes in the dock state,
    /// and the indices of containing surfaces.
    pub fn iter_all_nodes_mut(&mut self) -> impl Iterator<Item = (SurfaceIndex, &mut Node<Tab>)> {
        self.iter_surfaces_mut()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_nodes_mut()
                    .map(move |node| (surface_index.into(), node))
            })
    }

    /// Returns an [`Iterator`] of **all** tabs in the dock state,
    /// and the indices of containing surfaces and nodes.
    pub fn iter_all_tabs(&self) -> impl Iterator<Item = ((SurfaceIndex, NodeIndex), &Tab)> {
        self.iter_surfaces()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_all_tabs()
                    .map(move |(node_index, tab)| ((surface_index.into(), node_index), tab))
            })
    }

    /// Returns a mutable [`Iterator`] of **all** tabs in the dock state,
    /// and the indices of containing surfaces and nodes.
    pub fn iter_all_tabs_mut(
        &mut self,
    ) -> impl Iterator<Item = ((SurfaceIndex, NodeIndex), &mut Tab)> {
        self.iter_surfaces_mut()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_all_tabs_mut()
                    .map(move |(node_index, tab)| ((surface_index.into(), node_index), tab))
            })
    }

    /// Returns an immutable [`Iterator`] of all [``LeafNode``]s in the dock state.
    pub fn iter_leaves(&self) -> impl Iterator<Item = (SurfaceIndex, &LeafNode<Tab>)> {
        self.iter_all_nodes()
            .filter_map(|(index, node)| node.leaf().map(|leaf| (index, leaf)))
    }

    /// Returns a mutable [`Iterator`] of all [``LeafNode``]s in the dock state.
    pub fn iter_leaves_mut(&mut self) -> impl Iterator<Item = (SurfaceIndex, &mut LeafNode<Tab>)> {
        self.iter_all_nodes_mut()
            .filter_map(|(index, node)| node.leaf_mut().map(|leaf| (index, leaf)))
    }

    /// Find a tab based on the conditions of a functino.
    ///
    /// Returns in which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_tab_from(
        &self,
        predicate: impl Fn(&Tab) -> bool,
    ) -> Option<(SurfaceIndex, NodeIndex, TabIndex)> {
        self.valid_surface_indices_iter().find_map(|surface_index| {
            self[surface_index]
                .find_tab_from(&predicate)
                .map(|(node_index, tab_index)| (surface_index, node_index, tab_index))
        })
    }

    /// Returns a new [`DockState`] while mapping and filtering the tab type.
    /// Any remaining empty [`Node`]s and [`Surface`]s are removed.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let dock_state = DockState::new(vec![1, 2, 3]);
    /// let mapped_dock_state = dock_state.filter_map_tabs(|tab| (tab % 2 == 1).then(|| tab.to_string()));
    ///
    /// let tabs: Vec<_> = mapped_dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["1".to_string(), "3".to_string()]);
    /// ```
    pub fn filter_map_tabs<F, NewTab>(&self, mut function: F) -> DockState<NewTab>
    where
        F: FnMut(&Tab) -> Option<NewTab>,
    {
        DockState {
            surfaces: self
                .surfaces
                .iter()
                .filter_map(|surface| {
                    let surface = surface.filter_map_tabs(&mut function);
                    (!surface.is_empty()).then_some(surface)
                })
                .collect(),
            focused_surface: self.focused_surface,
            translations: self.translations.clone(),
        }
    }

    /// Returns a new [`DockState`] while mapping the tab type.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let dock_state = DockState::new(vec![1, 2, 3]);
    /// let mapped_dock_state = dock_state.map_tabs(|tab| tab.to_string());
    ///
    /// let tabs: Vec<_> = mapped_dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    /// ```
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> DockState<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`DockState`] while filtering the tab type.
    /// Any remaining empty [`Node`]s and [`Surface`]s are removed.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let dock_state = DockState::new(["tab1", "tab2", "outlier"].map(str::to_string).to_vec());
    /// let filtered_dock_state = dock_state.filter_tabs(|tab| tab.starts_with("tab"));
    ///
    /// let tabs: Vec<_> = filtered_dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["tab1".to_string(), "tab2".to_string()]);
    /// ```
    pub fn filter_tabs<F>(&self, mut predicate: F) -> DockState<Tab>
    where
        F: FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// Any remaining empty [`Node`]s and [`Surface`]s are also removed.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let mut dock_state = DockState::new(["tab1", "tab2", "outlier"].map(str::to_string).to_vec());
    /// dock_state.retain_tabs(|tab| tab.starts_with("tab"));
    ///
    /// let tabs: Vec<_> = dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["tab1".to_string(), "tab2".to_string()]);
    /// ```
    pub fn retain_tabs<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        let mut main_surface = true;
        self.surfaces.retain_mut(|surface| {
            surface.retain_tabs(&mut predicate);
            std::mem::take(&mut main_surface) || !surface.is_empty()
        });
    }
}

impl<Tab: PartialEq> DockState<Tab> {
    /// Find the given tab.
    ///
    /// Returns in which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    ///
    /// See also: [`find_main_surface_tab`](DockState::find_main_surface_tab)
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(SurfaceIndex, NodeIndex, TabIndex)> {
        self.find_tab_from(|tab| tab == needle_tab)
    }

    /// Find the given tab on the main surface.
    ///
    /// Returns which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_main_surface_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, TabIndex)> {
        self[SurfaceIndex::main()].find_tab(needle_tab)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn retain_none_then_push() {
        let mut t = DockState::new(vec![]);
        t.push_to_focused_leaf(0);
        let i = t.find_tab(&0).unwrap();
        t.remove_tab(i);
        t.retain_tabs(|_| false);
        t.push_to_focused_leaf(0);
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct Tab(u64);

    /// Checks that `retain` works after removing a node
    #[test]
    fn remove_and_retain() {
        let mut tree: Tree<Tab> = Tree::new(vec![]);
        tree.push_to_focused_leaf(Tab(0));
        let (n0, _t0) = tree.find_tab(&Tab(0)).unwrap();
        tree.split_below(n0, 0.5, vec![Tab(1)]);

        let i1 = tree.find_tab(&Tab(1)).unwrap();
        tree.remove_tab(i1);
        assert_eq!(tree.nodes.len(), 1);

        tree.retain_tabs(|_| true);
        assert!(tree.find_tab(&Tab(0)).is_some());
    }

    /// Tests whether `retain_tabs` works correctly with trailing `Empty` nodes
    #[test]
    fn retain_trailing_empty() {
        let mut tree: Tree<Tab> = Tree::new(vec![]);
        tree.push_to_focused_leaf(Tab(0));
        tree.nodes.push(Node::Empty);
        tree.nodes.push(Node::Empty);

        tree.retain_tabs(|_| true);
        assert!(tree.find_tab(&Tab(0)).is_some());
    }
}
