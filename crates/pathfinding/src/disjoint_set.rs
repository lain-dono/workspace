use bevy::platform::collections::HashMap;

/// A disjoint-set data structure for tracking which elements are joined, without managing any additional data associated to the elements.
///
/// This structure has methods like [`join`] or [`is_joined`] to modify or query which data is joined to which. For all of these, the elements are identified with their corresponding index. A `DisjointSet` of [`len`] `n` tracks elements from `0` to `n - 1`.
///
/// [`join`]: DisjointSet::join
/// [`is_joined`]: DisjointSet::is_joined
/// [`len`]: DisjointSet::len
///
/// # Examples
///
/// ```
/// use pathfinding::disjoint_set::DisjointSet;
///
/// // Initially, elements are totally disjoint.
/// let mut ds = DisjointSet::with_len(3); // {0}, {1}, {2}
/// assert!(ds.is_joined(0, 0));
/// assert!(!ds.is_joined(0, 1));
///
/// // Elements can be joined together
/// ds.join(0, 1); // {0, 1}, {2}
/// assert!(ds.is_joined(0, 1));
/// assert!(ds.is_joined(1, 0));
/// assert!(!ds.is_joined(0, 2));
///
/// // Since 0 was joined to 1, if we join 1 to 2, then 0 is joined to 2.
/// ds.join(1, 2); // {0, 1, 2}
/// assert!(ds.is_joined(0, 2));
/// ```
/// For a real word application example, see [the crate examples].
///
/// [the crate examples]: crate#examples
#[allow(clippy::missing_inline_in_public_items)]
#[derive(Debug, Clone)]
pub struct DisjointSet {
    parents: Vec<u32>,
    ranks: Vec<u8>,
}

impl Default for DisjointSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl DisjointSet {
    /// Returns an element of the subset containing `child`.
    /// This exact element is returned for every member of the subset.
    ///
    /// # Important
    ///
    /// The specific choice of the returned element is an implementation detail.
    /// There are no further guarantees beyond what is documented here.
    /// If you just want to check if two elements are in the same subset, use [`is_joined`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut ds = DisjointSet::with_len(3); // {0}, {1}, {2}
    /// assert_eq!(ds.root_of(0), 0);
    /// assert_eq!(ds.root_of(1), 1);
    /// assert_eq!(ds.root_of(2), 2);
    ///
    ///
    /// ds.join(0, 1); // {0, 1}, {2}
    /// assert_eq!(ds.root_of(0), ds.root_of(1));
    /// assert_ne!(ds.root_of(0), ds.root_of(2));
    ///
    /// ds.join(1, 2); // {0, 1, 2}
    /// assert_eq!(ds.root_of(0), ds.root_of(1));
    /// assert_eq!(ds.root_of(0), ds.root_of(2));
    /// ```
    ///
    /// [`is_joined`]: DisjointSet::is_joined
    #[inline]
    #[must_use]
    pub fn root_of(&mut self, mut child: u32) -> u32 {
        let mut parent = self.parents[child as usize];
        if parent == child {
            return child;
        }

        loop {
            let grandparent = self.parents[parent as usize];
            if parent == grandparent {
                return parent;
            }

            self.parents[child as usize] = grandparent;
            child = parent;
            parent = grandparent;
        }
    }

    /// Constructs a new `DisjointSet` with `len` elements, named `0` to `n - 1`, each in its own set.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut ds = DisjointSet::with_len(4);
    ///
    /// // The disjoint set contains 4 elements.
    /// assert_eq!(ds.len(), 4);
    ///
    /// // Two elements i and j are not joined in the same set, unless i = j.
    /// assert!(!ds.is_joined(0, 3));
    /// assert!(ds.is_joined(1, 1));
    ///
    /// ```
    #[inline]
    #[must_use]
    pub fn with_len(len: usize) -> Self {
        Self {
            parents: (0..len as u32).collect(),
            ranks: vec![0; len],
        }
    }

    /// Constructs a new, empty `DisjointSet` with at least the specified capacity.
    ///
    /// It will be able to hold at least `capacity` elements without
    /// reallocating. This method is allowed to allocate for more elements than
    /// `capacity`. If `capacity` is 0, it will not allocate.
    ///
    /// It is important to note that although the returned `DisjointSet` has the
    /// minimum *capacity* specified, it will have a zero *length*.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut ds = DisjointSet::with_capacity(10);
    ///
    /// // It contains no elements, even though it has capacity for more.
    /// assert_eq!(ds.len(), 0);
    ///
    /// // These are all done without reallocating...
    /// for _ in 0..10 {
    ///     ds.add_singleton();
    /// }
    ///
    /// // ...but this may make the disjoint set reallocate.
    /// ds.add_singleton();
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            parents: Vec::with_capacity(capacity),
            ranks: Vec::with_capacity(capacity),
        }
    }

    /// Adds a new element, not joined to any other element. Returns the index
    /// of the new element.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut ds = DisjointSet::with_len(1);
    /// assert_eq!(ds.add_singleton(), 1);
    /// assert_eq!(ds.len(), 2);
    /// assert!(!ds.is_joined(0, 1));
    /// ```
    #[inline]
    pub fn add_singleton(&mut self) -> u32 {
        let id = self.len() as u32;
        self.parents.push(id);
        self.ranks.push(0);
        id
    }

    /// If `first_element` and `second_element` are in different sets, joins them together and returns `true`.
    ///
    /// Otherwise, does nothing and returns `false`.
    ///
    /// # Panics
    ///
    /// Panics if `first_element` or `second_element` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// // Initially, each element is in its own set.
    /// let mut ds = DisjointSet::with_len(4); // {0}, {1}, {2}, {3}
    /// assert!(!ds.is_joined(0, 3));
    ///
    /// // By joining 0 to 1 and 2 to 3, we get two sets of two elements each.
    /// ds.join(0, 1); // {0, 1}, {2}, {3}
    /// ds.join(2, 3); // {0, 1}, {2, 3}
    /// assert!(ds.is_joined(0, 1));
    /// assert!(ds.is_joined(2, 3));
    /// assert!(!ds.is_joined(0, 3));
    ///
    /// // By further joining 2 to 3, all elements are now in the same set.
    /// ds.join(1, 2); // {0, 1, 2, 3}
    /// assert!(ds.is_joined(0, 3));
    /// ```
    #[inline]
    pub fn join(&mut self, first: u32, second: u32) -> bool {
        // Immediate parent check.
        if self.parents[first as usize] == self.parents[second as usize] {
            return false;
        }

        self.slow_path(first, second)
    }

    fn slow_path(&mut self, first: u32, second: u32) -> bool {
        let root_first = self.root_of(first);
        let root_second = self.root_of(second);

        if root_first == root_second {
            return false;
        }

        let rank_second = self.ranks[root_second as usize];
        let rank_first = &mut self.ranks[root_first as usize];

        if *rank_first < rank_second {
            self.parents[root_first as usize] = root_second;
        } else {
            if *rank_first == rank_second {
                *rank_first += 1;
            }
            self.parents[root_second as usize] = root_first;
        }

        true
    }

    /// Returns `true` if `first_element` and `second_element` are in the same subset.
    ///
    /// # Panics
    ///
    /// Panics if `first_element` or `second_element` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// // Initially, elements are only joined to themselves.
    /// let mut ds = DisjointSet::with_len(3); // {0}, {1}, {2}
    /// assert!(ds.is_joined(0, 0));
    /// assert!(!ds.is_joined(0, 1));
    /// assert!(!ds.is_joined(0, 2));
    ///
    /// // By joining 1 to 0, we implicitely join 0 to 1.
    /// ds.join(1, 0); // {0, 1}, {2}
    /// assert!(ds.is_joined(1, 0));
    /// assert!(ds.is_joined(0, 1));
    ///
    /// // By joining 0 to 1 and 1 to 2, we implicitely join 0 to 2.
    /// ds.join(1, 2); // {0, 1, 2}
    /// assert!(ds.is_joined(0, 2));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_joined(&mut self, first: u32, second: u32) -> bool {
        self.root_of(first) == self.root_of(second)
    }

    /// Returns the number of elements in the disjoint set, regardless of how they are joined together.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut ds = DisjointSet::with_len(4);
    /// assert_eq!(ds.len(), 4);
    ///
    /// ds.join(1, 3);
    /// assert_eq!(ds.len(), 4);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.parents.len()
    }

    /// Returns `true` if the disjoint set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// assert!(DisjointSet::new().is_empty());
    /// assert!(DisjointSet::with_len(0).is_empty());
    /// assert!(!DisjointSet::with_len(10).is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }

    /// Constructs a new, empty `DisjointSet`.
    ///
    /// The disjoint set will not allocate until elements are added to it.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            parents: Vec::new(),
            ranks: Vec::new(),
        }
    }

    /// Clears the `DisjointSet`.
    ///
    /// The disjoint set will retain its capacity, so adding elements will not
    /// allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut set = DisjointSet::new();
    /// set.add_singleton();
    /// set.add_singleton();
    /// set.clear();
    /// assert_eq!(set.len(), 0);
    /// // Does not allocate!
    /// set.add_singleton();
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.parents.clear();
        self.ranks.clear();
    }

    /// Returns a `Vec` of all sets. Each entry corresponds to one set, and is a `Vec` of its elements.
    ///
    /// The sets are ordered by their smallest contained element. The elements inside each sets are ordered.
    ///
    /// # Examples
    ///
    /// ```
    /// use pathfinding::disjoint_set::DisjointSet;
    ///
    /// let mut ds = DisjointSet::with_len(4); // {0}, {1}, {2}, {3}
    /// ds.join(3, 1); // {0}, {1, 3}, {2}
    /// assert_eq!(ds.sets(), vec![vec![0], vec![1, 3], vec![2]]);
    /// ```
    #[must_use]
    #[allow(clippy::missing_inline_in_public_items)]
    pub fn sets(&mut self) -> Vec<Vec<u32>> {
        let mut result = Vec::new();
        let mut root_to_result_id = HashMap::new();

        for index in 0..self.len() as u32 {
            let root = self.root_of(index);
            let &mut result_id = root_to_result_id.entry(root).or_insert_with(|| {
                let id = result.len();
                result.push(Vec::with_capacity(1));
                id
            });
            result[result_id].push(index);
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::DisjointSet;

    #[test]
    fn join_returns_false_even_if_immediate_parent_check_fails() {
        let mut ds = DisjointSet::with_len(4);

        ds.join(0, 1);
        ds.join(2, 3);
        ds.join(2, 0);

        assert_ne!(ds.parents[1], ds.parents[3]);
        assert!(!ds.join(1, 3));
    }

    #[test]
    fn clear_removes_elements_without_removing_capacity() {
        let mut set = DisjointSet::new();
        set.add_singleton();
        set.add_singleton();
        let capacity = set.parents.capacity();
        set.clear();
        assert_eq!(set.parents.capacity(), capacity);
    }
}
