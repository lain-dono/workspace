use bevy::math::Vec3;

#[derive(Default, Debug)]
pub struct NavMeshBuilder {
    verts: Vec<VertSlot>,
    links: Vec<LinkSlot>,
    faces: Vec<FaceSlot>,
}

impl std::ops::Index<Vert> for NavMeshBuilder {
    type Output = VertSlot;
    fn index(&self, index: Vert) -> &Self::Output {
        &self.verts[index.get()]
    }
}

impl std::ops::IndexMut<Vert> for NavMeshBuilder {
    fn index_mut(&mut self, index: Vert) -> &mut Self::Output {
        &mut self.verts[index.get()]
    }
}

impl std::ops::Index<Link> for NavMeshBuilder {
    type Output = LinkSlot;
    fn index(&self, index: Link) -> &Self::Output {
        &self.links[index.get()]
    }
}

impl std::ops::IndexMut<Link> for NavMeshBuilder {
    fn index_mut(&mut self, index: Link) -> &mut Self::Output {
        &mut self.links[index.get()]
    }
}

impl std::ops::Index<Face> for NavMeshBuilder {
    type Output = FaceSlot;
    fn index(&self, index: Face) -> &Self::Output {
        &self.faces[index.get()]
    }
}

impl NavMeshBuilder {
    pub fn all_boundary(&self) -> impl Iterator<Item = Link> {
        (0..self.links.len()).filter_map(|index| {
            let link = Link::new(index).unwrap();
            self.link_is_boundary(link).then_some(link)
        })
    }

    // ccw
    fn iter_prev_twin(&self, base: Link) -> LinkIter<impl Fn(Link) -> Link> {
        base.iter(|iter| self[self[iter].prev].twin)
    }

    // cw
    fn iter_twin_next(&self, base: Link) -> LinkIter<impl Fn(Link) -> Link> {
        base.iter(|iter| self[self[iter].twin].next)
    }

    fn iter_next(&self, base: Link) -> LinkIter<impl Fn(Link) -> Link> {
        base.iter(|iter| self[iter].next)
    }

    fn iter_prev(&self, base: Link) -> LinkIter<impl Fn(Link) -> Link> {
        base.iter(|iter| self[iter].prev)
    }

    fn link_is_free(&self, link: Link) -> bool {
        self[link].face.is_none()
    }

    fn link_is_boundary(&self, link: Link) -> bool {
        self[link].face.is_none() && self[self[link].twin].face.is_some()
    }

    fn vert_is_isolated(&self, vert: Vert) -> bool {
        self[vert].link.is_none()
    }

    fn vert_is_free(&self, vert: Vert) -> bool {
        self[vert].link.is_none_or(|base| {
            self.iter_twin_next(base)
                .any(|link| self.link_is_free(link))
        })
    }

    fn find_link_to(&self, from: Vert, to: Vert) -> Option<Link> {
        self[from].link.and_then(|base| {
            self.iter_twin_next(base)
                .find(|&link| self[self[link].twin].vert == to)
        })
    }

    fn free_twins(&self, base: Link) -> impl Iterator<Item = Link> {
        self.iter_twin_next(base).filter_map(|link| {
            let link = self[link].twin;
            self.link_is_free(link).then_some(link)
        })
    }

    pub fn add_vert(&mut self, position: Vec3) -> Vert {
        let index = Vert::new(self.verts.len()).unwrap();
        let link = None;
        self.verts.push(VertSlot { link, position });
        index
    }

    pub fn add_edge(&mut self, v1: Vert, v2: Vert) -> [Link; 2] {
        assert_ne!(v1, v2, "vertices should be different");

        // Check if v1 and v2 are already connected
        let a = self.find_link_to(v1, v2);
        let b = self.find_link_to(v2, v1);
        match [a, b] {
            [Some(a), Some(b)] => return [a, b],
            [None, None] => (),
            _ => panic!("broken connectivity"),
        }

        assert!(self.vert_is_free(v1), "v1 should be free");
        assert!(self.vert_is_free(v2), "v2 should be free");

        // Create new halfedges, by default twin halfedges are connected together
        // as prev/next in case vertices are isolated

        let new1 = Link::new(self.links.len()).unwrap();
        let new2 = Link::new(self.links.len() + 1).unwrap();

        let mut slot1 = LinkSlot {
            vert: v1,
            face: None,
            twin: new2,
            next: new2,
            prev: new2,
        };

        let mut slot2 = LinkSlot {
            vert: v2,
            face: None,
            twin: new1,
            next: new1,
            prev: new1,
        };

        // Update refs around v1 if not isolated
        if let Some(prev) = self[v1].link.and_then(|base| self.free_twins(base).next()) {
            let next = self[prev].next;
            slot1.prev = prev;
            slot2.next = next;
            self[prev].next = new1;
            self[next].prev = new2;
        } else {
            self[v1].link = Some(new1);
        }

        // Update refs around v2 if not isolated
        if let Some(prev) = self[v2].link.and_then(|base| self.free_twins(base).next()) {
            let next = self[prev].next;
            slot2.prev = prev;
            slot1.next = next;
            self[prev].next = new2;
            self[next].prev = new1;
        } else {
            self[v2].link = Some(new2);
        }

        self.links.push(slot1);
        self.links.push(slot2);

        [new1, new2]
    }

    pub fn add_face(&mut self, links: &[Link]) -> Face {
        assert!(links.len() >= 3, "at least 3 links required");

        // Make some checks before changing topology
        for i in 0..links.len() {
            let [prev, next] = [links[i], links[(i + 1) % links.len()]];
            assert!(self[prev].face.is_none(), "link should be free");

            let twin_vert = self[self[prev].twin].vert;
            assert_eq!(twin_vert, self[next].vert, "links do not form a chain");
        }

        // Add the face
        for i in 0..links.len() {
            let [prev, next] = [links[i], links[(i + 1) % links.len()]];
            let adjacent = self.make_halfedges_adjacent(prev, next);
            assert!(adjacent, "mesh should be manifold");
        }

        let index = Face::new(self.faces.len()).unwrap();
        for &link in links {
            self[link].face = Some(index);
        }

        let link = links[0];
        self.faces.push(FaceSlot { link });
        index
    }

    // see https://kaba.hilvi.org/homepage/blog/halfedge/halfedge.htm
    fn make_halfedges_adjacent(&mut self, prev: Link, next: Link) -> bool {
        if self[prev].next == next {
            // Adjacency is alrady correct
            return true;
        }

        // Find a boundary halfedge different from next.twin and prev

        let link = self[self[next].vert].link;
        let Some(free) = link.and_then(|base| {
            let iter = self.free_twins(base);
            iter.filter(|&link| link != prev).last()
        }) else {
            return false;
        };

        let prev_next = self[prev].next;
        let next_prev = self[next].prev;
        let free_next = self[free].next;

        self[prev].next = next;
        self[next].prev = prev;
        self[free].next = prev_next;
        self[prev_next].prev = free;
        self[next_prev].next = free_next;
        self[free_next].prev = next_prev;

        true
    }
}

pub struct Iter<T, F> {
    base: T,
    iter: Option<T>,
    next: F,
}

impl<T: Copy + PartialEq, F: Fn(T) -> T> Iter<T, F> {
    fn new(base: T, next: F) -> Self {
        let iter = None;
        Self { base, iter, next }
    }
}

impl<T: Copy + PartialEq, F: Fn(T) -> T> Iterator for Iter<T, F> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = self.iter {
            self.iter = Some((self.next)(iter));
            self.iter.filter(|&iter| iter != self.base)
        } else {
            self.iter = Some(self.base);
            self.iter
        }
    }
}

#[derive(Debug)]
pub struct VertSlot {
    pub link: Option<Link>,
    pub position: Vec3,
}

#[derive(Debug)]
pub struct LinkSlot {
    pub vert: Vert,
    pub face: Option<Face>,
    pub twin: Link,
    pub prev: Link,
    pub next: Link,
}

#[derive(Debug)]
pub struct FaceSlot {
    pub link: Link,
}

macro_rules! impl_nonmax32 {
    ($name:ident, $iter:ident) => {
        impl_nonmax32!($name);

        pub type $iter<F> = Iter<$name, F>;

        impl $name {
            pub fn iter<F: Fn($name) -> $name>(self, next: F) -> $iter<F> {
                $iter::new(self, next)
            }
        }
    };

    ($name:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[must_use]
        pub struct $name(core::num::NonZeroU32);

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.get().fmt(f)
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.get().fmt(f)
            }
        }

        impl $name {
            /// Construct a [`$name`] whose value is `n`, if possible.
            #[must_use]
            pub const fn new(n: usize) -> Option<Self> {
                if n >= u32::MAX as usize {
                    return None;
                }

                let n = n as u32;

                // If `n` is `u32::MAX`, then `n.wrapping_add(1)` is `0`,
                // so `NonZeroU32::new` returns `None` in exactly the case
                // where we must return `None`.
                match core::num::NonZeroU32::new(n.wrapping_add(1)) {
                    Some(non_zero) => Some(Self(non_zero)),
                    None => None,
                }
            }

            /// Return the value of `self` as a [`u32`].
            #[must_use]
            pub const fn get(self) -> usize {
                (self.0.get() - 1) as usize
            }

            /// Construct a [`$name`] whose value is `n`.
            ///
            /// # Safety
            ///
            /// The value of `n` must not be [`u32::MAX`].
            pub const unsafe fn new_unchecked(n: u32) -> Self {
                Self(unsafe { core::num::NonZeroU32::new_unchecked(n + 1) })
            }

            /// Construct a [`$name`] whose value is `index`.
            ///
            /// # Safety
            ///
            /// - The value of `index` must be strictly less than [`u32::MAX`].
            pub const unsafe fn from_usize_unchecked(index: usize) -> Self {
                Self(unsafe { core::num::NonZeroU32::new_unchecked(index as u32 + 1) })
            }

            #[must_use]
            pub fn checked_add(self, n: u32) -> Option<Self> {
                // Adding `n` to `self` produces `u32::MAX` if and only if
                // adding `n` to `self.0` produces `0`. So we can simply
                // call `NonZeroU32::checked_add` and let its check for zero
                // determine whether our add would have produced `u32::MAX`.
                Some(Self(self.0.checked_add(n)?))
            }
        }
    };
}

impl_nonmax32!(Vert);
impl_nonmax32!(Link, LinkIter);
impl_nonmax32!(Face);

#[test]
fn size() {
    assert_eq!(size_of::<Option<Vert>>(), size_of::<u32>());
    assert_eq!(size_of::<Option<Link>>(), size_of::<u32>());
    assert_eq!(size_of::<Option<Face>>(), size_of::<u32>());
}

#[test]

fn add_edge() {
    let mut mesh = NavMeshBuilder::default();

    //       v2
    //       | \
    //       |   \
    //       |     \
    //      v0 ----- v1

    let v0 = mesh.add_vert(Vec3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vert(Vec3::new(2.0, 0.0, 0.0));
    let v2 = mesh.add_vert(Vec3::new(0.0, 2.0, 0.0));

    println!("Link isolated vertices");
    let [v0v1, v1v0] = mesh.add_edge(v0, v1);
    assert_eq!(mesh[v0v1].twin, v1v0);

    assert_eq!(mesh[v0v1].next, v1v0);
    assert_eq!(mesh[v0v1].prev, v1v0);
    assert_eq!(mesh[v1v0].next, v0v1);
    assert_eq!(mesh[v1v0].prev, v0v1);
    assert_eq!(mesh[v0].link, Some(v0v1));
    assert_eq!(mesh[v1].link, Some(v1v0));

    println!("Link to another edge");
    let [v1v2, v2v1] = mesh.add_edge(v1, v2);
    assert_eq!(mesh[v1v2].twin, v2v1);

    assert_eq!(mesh[v1v2].next, v2v1);
    assert_eq!(mesh[v1v2].prev, v0v1);
    assert_eq!(mesh[v0v1].next, v1v2);
    assert_eq!(mesh[v2v1].next, v1v0);
    assert_eq!(mesh[v2v1].prev, v1v2);
    assert_eq!(mesh[v1v0].prev, v2v1);

    println!("Closing a loop");
    let [v2v0, v0v2] = mesh.add_edge(v2, v0);
    assert_eq!(mesh[v2v0].twin, v0v2);

    assert_eq!(mesh[v2v0].next, v0v1);
    assert_eq!(mesh[v2v0].prev, v1v2);
    assert_eq!(mesh[v0v1].prev, v2v0);
    assert_eq!(mesh[v1v2].next, v2v0);

    assert_eq!(mesh[v0v2].next, v2v1);
    assert_eq!(mesh[v0v2].prev, v1v0);
    assert_eq!(mesh[v1v0].next, v0v2);
    assert_eq!(mesh[v2v1].prev, v0v2);

    //       v2      v3
    //       | \     | \
    //       |   \   |   \
    //       |     \ |     \
    //      v0 ---- v1 ---- v4

    let v3 = mesh.add_vert(Vec3::new(2.0, 2.0, 0.0));
    let v4 = mesh.add_vert(Vec3::new(4.0, 2.0, 0.0));

    println!("Connect to face");
    let _ = mesh.add_face(&[v0v1, v1v2, v2v0]);

    let [v3v1, v1v3] = mesh.add_edge(v3, v1);

    assert_eq!(mesh[v3v1].next, v1v0);
    assert_eq!(mesh[v3v1].prev, v1v3);
    assert_eq!(mesh[v1v3].next, v3v1);
    assert_eq!(mesh[v1v3].prev, v2v1);

    let [v1v4, v4v1] = mesh.add_edge(v1, v4);

    assert_eq!(mesh[v1v4].next, v4v1);
    assert_eq!(mesh[v4v1].prev, v1v4);
    assert_eq!(mesh[v4v1].next, v1v3); // [v1v0, v1v3]
    assert_eq!(mesh[v1v4].prev, v2v1); // [v2v1, v3v1]

    let [v4v3, v3v4] = mesh.add_edge(v4, v3);

    assert_eq!(mesh[v4v3].next, v3v1);
    assert_eq!(mesh[v4v3].prev, v1v4);
    assert_eq!(mesh[v3v4].next, v4v1);
    assert_eq!(mesh[v3v4].prev, v1v3);

    let _ = mesh.add_face(&[v1v4, v4v3, v3v1]);

    assert_eq!(mesh[v1v4].prev, v3v1);
    assert_eq!(mesh[v3v1].next, v1v4);
}
