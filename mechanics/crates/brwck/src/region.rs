use crate::env::Point;
use std::collections::BTreeSet;
use std::fmt;

/// A region is a set of points where, within any given basic block,
/// the points must be continuous. We represent this as a map:
///
/// `B -> start..end`
///
/// where `B` is a basic block identifier and start/end are indices.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Region {
    points: BTreeSet<Point>,
}

impl Region {
    pub fn add_point(&mut self, point: Point) -> bool {
        self.points.insert(point)
    }

    pub fn may_contain(&self, point: Point) -> bool {
        self.points.contains(&point)
    }
}

impl fmt::Debug for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("{")?;
        for (index, point) in self.points.iter().enumerate() {
            match index {
                0 => write!(f, "{point:?}")?,
                _ => write!(f, ", {point:?}")?,
            }
        }
        f.write_str("}")
    }
}
