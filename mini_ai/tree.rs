use crate::bbox::BoundingBox;
use bevy::math::Vec3;
use ord_subset::OrdVar;

pub struct BoundingBoxHierarchy<T> {
    nodes: Vec<Option<Node<T>>>,
}

impl<T> std::ops::Index<NodeIndex> for BoundingBoxHierarchy<T> {
    type Output = Option<Node<T>>;

    #[inline]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<T> std::ops::IndexMut<NodeIndex> for BoundingBoxHierarchy<T> {
    #[inline]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<'a, T> IntoIterator for &'a BoundingBoxHierarchy<T> {
    type Item = &'a Option<Node<T>>;
    type IntoIter = std::slice::Iter<'a, Option<Node<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut BoundingBoxHierarchy<T> {
    type Item = &'a mut Option<Node<T>>;
    type IntoIter = std::slice::IterMut<'a, Option<Node<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

impl<T> BoundingBoxHierarchy<T> {
    #[must_use]
    pub fn new(root: Option<Node<T>>) -> Self {
        Self { nodes: vec![root] }
    }

    pub fn from_values(values: &mut [(BoundingBox, Option<T>)]) -> Self
    where
        T: Copy,
    {
        let len = values.len().next_power_of_two() * 2 - 1;
        let tree = vec![None; len];
        let mut tree = Self { nodes: tree };
        Self::build_recursive(&mut tree, NodeIndex::root(), values);
        tree
    }

    fn build_recursive(tree: &mut Self, index: NodeIndex, values: &mut [(BoundingBox, Option<T>)])
    where
        T: Copy,
    {
        assert!(!values.is_empty());
        if values.len() == 1 {
            let (bounds, value) = values[0];
            let value = value.unwrap();

            tree[index] = Some(Node::Leaf(bounds, value));

            return;
        }

        let bounds = BoundingBox::from_cloud(values.iter().map(|&(bbox, _)| bbox));

        let size = bounds.size();
        if size.x > size.y && size.x > size.z {
            values.sort_by_key(|(bbox, _)| OrdVar::new_unchecked(bbox.center().unwrap().x));
        } else if size.y > size.z {
            values.sort_by_key(|(bbox, _)| OrdVar::new_unchecked(bbox.center().unwrap().y));
        } else {
            values.sort_by_key(|(bbox, _)| OrdVar::new_unchecked(bbox.center().unwrap().z));
        }

        let (prev, next) = values.split_at_mut(values.len() / 2);

        tree[index] = Some(Node::Pair(bounds));
        Self::build_recursive(tree, index.prev(), prev);
        Self::build_recursive(tree, index.next(), next);
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        NodeIndex(self.len() - 1).level()
    }

    #[must_use]
    pub fn query_box(&self, min: Vec3, max: Vec3) -> Vec<T>
    where
        T: Copy,
    {
        self.query(BoundingBox::new(min, max))
    }

    #[must_use]
    pub fn query(&self, query: BoundingBox) -> Vec<T>
    where
        T: Copy,
    {
        let mut result = Vec::new();
        if !query.is_empty() {
            self.query_recursive(&query, NodeIndex::root(), &mut result);
        }
        result
    }

    fn query_recursive(&self, query: &BoundingBox, index: NodeIndex, result: &mut Vec<T>)
    where
        T: Copy,
    {
        match self[index].unwrap() {
            Node::Leaf(bounds, value) => {
                if query.intersects_bounds(&bounds) {
                    result.push(value);
                }
            }
            Node::Pair(bounds) => {
                if query.intersects_bounds(&bounds) {
                    self.query_recursive(query, index.prev(), result);
                    self.query_recursive(query, index.next(), result);
                }
            }
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Option<Node<T>>> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Option<Node<T>>> {
        self.nodes.iter_mut()
    }

    pub fn split(&mut self, node: NodeIndex, data: BoundingBox, new: (BoundingBox, T)) {
        assert!(matches!(self[node], Some(Node::Leaf(_, _))));

        let old = self[node].replace(Node::Pair(data));
        if self.len() <= node.next().0 {
            let new_len = self.nodes.len() * 2 + 1;
            self.nodes.resize_with(new_len, || None);
        }

        self[node.prev()] = old;
        self[node.next()] = Some(Node::Leaf(new.0, new.1));
    }

    pub fn remove_leaf(&mut self, node: NodeIndex) {
        assert!(matches!(self[node], Some(Node::Leaf(_, _))));

        let Some(parent) = node.parent() else {
            self.nodes.clear();
            return;
        };

        self[parent] = None;
        self[node] = None;

        let mut level = 0;

        if node.is_left() {
            'end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_next(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.nodes.len() {
                        break 'end;
                    }
                    self.nodes[dst] = self.nodes[src].take();
                }
                level += 1;
            }
        } else {
            'end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_prev(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.nodes.len() {
                        break 'end;
                    }
                    self.nodes[dst] = self.nodes[src].take();
                }
                level += 1;
            }
        }
    }

    #[must_use]
    pub fn first_leaf(&self, top: NodeIndex) -> Option<NodeIndex> {
        let prev = top.prev();
        let next = top.next();

        let prev_node = self.nodes.get(prev.0).and_then(Option::as_ref);
        let next_node = self.nodes.get(next.0).and_then(Option::as_ref);

        match (prev_node, next_node) {
            (Some(Node::Leaf(_, _)), _) => Some(prev),
            (_, Some(Node::Leaf(_, _))) => Some(next),
            (Some(Node::Pair(_)), Some(Node::Pair(_))) => {
                self.first_leaf(prev).or_else(|| self.first_leaf(next))
            }
            (Some(Node::Pair(_)), _) => self.first_leaf(prev),
            (_, Some(Node::Pair(_))) => self.first_leaf(next),
            (None, None) => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Node<T> {
    Leaf(BoundingBox, T),
    Pair(BoundingBox),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeIndex(pub usize);

impl NodeIndex {
    #[must_use]
    pub const fn root() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn new_len(self) -> usize {
        (1 << (self.level() + 1)) - 1
    }

    #[must_use]
    pub const fn prev(self) -> Self {
        Self(self.0 * 2 + 1)
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 * 2 + 2)
    }

    #[must_use]
    pub const fn parent(self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self((self.0 - 1) / 2))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn level(self) -> usize {
        (usize::BITS - (self.0 + 1).leading_zeros()) as usize
    }

    #[must_use]
    pub const fn is_left(self) -> bool {
        self.0 % 2 != 0
    }

    #[must_use]
    pub const fn is_right(self) -> bool {
        self.0 % 2 == 0
    }

    const fn children_at(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base - 1;
        let e = (self.0 + 2) * base - 1;
        s..e
    }

    const fn children_prev(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base - 1;
        let e = (self.0 + 1) * base + base / 2 - 1;
        s..e
    }

    const fn children_next(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base + base / 2 - 1;
        let e = (self.0 + 2) * base - 1;
        s..e
    }
}

#[test]
fn splitting() {
    use Node::{Leaf, Pair};

    let data = BoundingBox::empty();

    let [a, b, c, d] = [1, 2, 3, 4];
    let a_node = Leaf(data, a);
    let b_node = Leaf(data, b);
    let c_node = Leaf(data, c);
    let d_node = Leaf(data, d);

    let mut tree = BoundingBoxHierarchy::new(Some(a_node));
    assert_eq!(tree.depth(), 1);

    tree.split(NodeIndex::root(), data, (data, b));
    assert_eq!(tree.depth(), 2);
    assert_eq!(tree.nodes, [Some(Pair(data)), Some(a_node), Some(b_node)]);

    tree.split(NodeIndex(1), data, (data, c)); // split prev node
    assert_eq!(tree.depth(), 3);
    assert_eq!(
        tree.nodes,
        [
            // root
            Some(Pair(data)),
            // root prev/next
            Some(Pair(data)),
            Some(b_node),
            // prev prev/next
            Some(a_node),
            Some(c_node),
            // b_node prev/next
            None,
            None,
        ]
    );

    tree.split(NodeIndex(2), data, (data, d)); // split next node
    assert_eq!(tree.depth(), 3);
    assert_eq!(
        tree.nodes,
        [
            // root
            Some(Pair(data)),
            // root prev/next
            Some(Pair(data)),
            Some(Pair(data)),
            // prev prev/next
            Some(a_node),
            Some(c_node),
            // next prev/next
            Some(b_node),
            Some(d_node),
        ]
    );
}

#[test]
fn octant_hierarchy() {
    use bevy::math::Vec3;

    let mut values = [
        (0, Vec3::new(1.0, 1.0, 1.0), Vec3::new(4.0, 4.0, 4.0)),
        (1, Vec3::new(5.0, 1.0, 1.0), Vec3::new(8.0, 4.0, 4.0)),
        (2, Vec3::new(1.0, 5.0, 1.0), Vec3::new(4.0, 8.0, 4.0)),
        (3, Vec3::new(5.0, 5.0, 1.0), Vec3::new(8.0, 8.0, 4.0)),
        (4, Vec3::new(1.0, 1.0, 5.0), Vec3::new(4.0, 4.0, 8.0)),
        (5, Vec3::new(5.0, 1.0, 5.0), Vec3::new(8.0, 4.0, 8.0)),
        (6, Vec3::new(1.0, 5.0, 5.0), Vec3::new(4.0, 8.0, 8.0)),
        (7, Vec3::new(5.0, 5.0, 5.0), Vec3::new(8.0, 8.0, 8.0)),
    ]
    .map(|(index, min, max)| (BoundingBox::new(min, max), Some(index)));

    let bbh = BoundingBoxHierarchy::from_values(&mut values);

    assert_eq!(bbh.depth(), 4);

    assert!(bbh.query(BoundingBox::empty()).is_empty());
    let result = bbh.query_box(Vec3::new(1.0, 5.0, 1.0), Vec3::new(4.0, 8.0, 4.0));
    assert_eq!(result, [2]);
    let result = bbh.query_box(Vec3::new(1.0, 5.0, 1.0), Vec3::new(6.0, 8.0, 4.0));
    assert_eq!(result, [2, 3]);
    let result = bbh.query_box(Vec3::new(2.0, 2.0, 2.0), Vec3::new(7.0, 7.0, 7.0));
    assert_eq!(result, [0, 1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn hierarchy_with_same_big_dimension() {
    use bevy::math::Vec3;

    let mut values = [
        (0, Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 11.0)),
        (1, Vec3::new(4.0, 1.0, 1.0), Vec3::new(5.0, 2.0, 11.0)),
    ]
    .map(|(index, min, max)| (BoundingBox::new(min, max), Some(index)));

    let bbh = BoundingBoxHierarchy::from_values(&mut values);
    let result = bbh.query_box(Vec3::new(1.5, 1.5, 1.5), Vec3::new(1.5, 1.5, 1.5));
    assert_eq!(result, [0]);
}
