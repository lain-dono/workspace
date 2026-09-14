use super::Tree;
use bevy::ecs::entity::Entity;

#[derive(Clone, Debug)]
pub struct Tab {
    pub icon: char,
    pub title: String,
    pub entity: Entity,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  {}", self.icon, self.title)
    }
}

impl Tab {
    pub fn new(icon: char, title: impl Into<String>, entity: Entity) -> Self {
        Self {
            icon,
            title: title.into(),
            entity,
        }
    }
}

/// Iterates over all tabs in a [`Tree`].
pub struct TabIter<'a, Tab> {
    tree: &'a Tree<Tab>,
    node_index: usize,
    tab_index: usize,
}

impl<Tab> std::fmt::Debug for TabIter<'_, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabIter").finish_non_exhaustive()
    }
}

impl<'a, Tab> TabIter<'a, Tab> {
    pub(super) fn new(tree: &'a Tree<Tab>) -> Self {
        Self {
            tree,
            node_index: 0,
            tab_index: 0,
        }
    }
}

impl<'a, Tab> Iterator for TabIter<'a, Tab> {
    type Item = &'a Tab;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.tree.nodes.get(self.node_index)?;
            if let Some(tab) = node.tabs().and_then(|tabs| tabs.get(self.tab_index)) {
                self.tab_index += 1;
                return Some(tab);
            }
            self.node_index += 1;
            self.tab_index = 0;
        }
    }
}

#[test]
fn test_tabs_iter() {
    fn tabs(tree: &Tree<i32>) -> Vec<i32> {
        tree.tabs().copied().collect()
    }

    let mut tree = Tree::new(vec![1, 2, 3]);
    assert_eq!(tabs(&tree), vec![1, 2, 3]);

    tree.push_to_first_leaf(4);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4]);

    tree.push_to_first_leaf(5);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4, 5]);

    tree.push_to_focused_leaf(6);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4, 5, 6]);

    assert_eq!(tree.num_tabs(), tree.tabs().count());
}
