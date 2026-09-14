use crate::{BMeshData, EdgeKey, EdgeSlot, Slots, VertKey};

pub struct DiskIter<'a, T: BMeshData> {
    data: &'a Slots<EdgeKey, EdgeSlot<T>>,
    edge: EdgeKey,
    vert: VertKey,
    iter: Option<EdgeKey>,
}

impl<'a, T: BMeshData> DiskIter<'a, T> {
    pub fn new(data: &'a Slots<EdgeKey, EdgeSlot<T>>, vert: VertKey, edge: EdgeKey) -> Self {
        Self {
            data,
            edge,
            vert,
            iter: None,
        }
    }
}

impl<T: BMeshData> Iterator for DiskIter<'_, T> {
    type Item = EdgeKey;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter {
            Some(edge) => {
                self.iter = self.data[edge].disk(self.vert).map(|disk| disk.next);
                self.iter.filter(|&e| e != self.edge)
            }
            None => {
                self.iter = Some(self.edge);
                self.iter
            }
        }
    }
}

impl<T: BMeshData> DoubleEndedIterator for DiskIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter {
            Some(edge) => {
                self.iter = self.data[edge].disk(self.vert).map(|disk| disk.prev);
                self.iter.filter(|&e| e != self.edge)
            }
            None => {
                self.iter = Some(self.edge);
                self.iter
            }
        }
    }
}
