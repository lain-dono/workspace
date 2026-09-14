use crate::{BMesh, BMeshData, FaceKey, LinkKey, LinkSlot, Slots};

pub struct LinkIter<'a, T: BMeshData> {
    links: &'a Slots<LinkKey, LinkSlot<T>>,
    link: LinkKey,
    iter: Option<LinkKey>,
}

impl<'a, T: BMeshData> LinkIter<'a, T> {
    pub fn face(mesh: &'a BMesh<T>, face: FaceKey) -> Self {
        Self::new(&mesh.links, mesh.faces[face].link)
    }

    pub fn new(links: &'a Slots<LinkKey, LinkSlot<T>>, link: LinkKey) -> Self {
        Self {
            links,
            link,
            iter: None,
        }
    }
}

impl<T: BMeshData> Iterator for LinkIter<'_, T> {
    type Item = LinkKey;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter {
            Some(link) => {
                self.iter = Some(self.links[link].face_next);
                self.iter.filter(|&link| link != self.link)
            }
            None => {
                self.iter = Some(self.link);
                self.iter
            }
        }
    }
}

impl<T: BMeshData> std::iter::DoubleEndedIterator for LinkIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter {
            Some(link) => {
                self.iter = Some(self.links[link].face_prev);
                self.iter.filter(|&link| link != self.link)
            }
            None => {
                self.iter = Some(self.link);
                self.iter
            }
        }
    }
}
