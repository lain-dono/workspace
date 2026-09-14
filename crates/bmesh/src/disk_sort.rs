use crate::{BMesh, BMeshData, DiskIter, EdgeKey, EdgeSlot, Slots, VertKey};

impl<T: BMeshData> BMesh<T> {
    pub fn sort_disk_all<F>(&mut self, order: &mut Vec<EdgeKey>, mut compare: F)
    where
        F: FnMut(&Self, VertKey, &EdgeSlot<T>, &EdgeSlot<T>) -> std::cmp::Ordering,
    {
        for (vert, _) in &self.verts {
            if let Some(edge) = self.verts.get(vert).and_then(|v| v.edge) {
                order.clear();
                order.extend(DiskIter::new(&self.edges, vert, edge));
                order
                    .sort_unstable_by(|&a, &b| compare(self, vert, &self.edges[a], &self.edges[b]));
                set_disk_order(&mut self.edges, vert, order);
            }
        }
    }
    pub fn sort_disk<F>(&mut self, order: &mut Vec<EdgeKey>, vert: VertKey, mut compare: F)
    where
        F: FnMut(&EdgeSlot<T>, &EdgeSlot<T>) -> std::cmp::Ordering,
    {
        if let Some(edge) = self.verts.get(vert).and_then(|v| v.edge) {
            order.clear();
            order.extend(DiskIter::new(&self.edges, vert, edge));
            order.sort_unstable_by(|&a, &b| compare(&self.edges[a], &self.edges[b]));
            set_disk_order(&mut self.edges, vert, order);
        }
    }
}

fn set_disk_order<T: BMeshData>(
    edges: &mut Slots<EdgeKey, EdgeSlot<T>>,
    vert: VertKey,
    order: &[EdgeKey],
) {
    if order.len() > 1 {
        for (current, &edge) in order.iter().enumerate() {
            let disk = &mut edges[edge][vert];
            disk.prev = order[(current + order.len() - 1) % order.len()];
            disk.next = order[(current + 1) % order.len()];
        }
    }
}
