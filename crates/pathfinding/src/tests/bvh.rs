use crate::bvh::{BoundingBox, BoundingBoxHierarchy, Transform};
use crate::landmass::XYZ;
use bevy::math::Vec3;
use std::f32::consts::PI;

#[test]
fn from_point_cloud() {
    let empty: [Vec3; 0] = [];
    let bbox = BoundingBox::from_point_cloud(empty);
    assert!(bbox.is_empty());

    let first = Vec3::new(3.0, 5.0, -10.0);
    let bbox = BoundingBox::from_point_cloud([first]);
    assert_eq!(bbox.to_min_max(), (first, first));

    // Other corner of box.
    let second = Vec3::new(1.0, 7.0, -8.0);

    let bbox = BoundingBox::from_point_cloud([first, second]);
    assert_eq!(
        bbox.to_min_max(),
        (Vec3::new(1.0, 5.0, -10.0), Vec3::new(3.0, 7.0, -8.0))
    );

    let max_bound = Vec3::new(30.0, 30.0, 30.0);
    let bbox = BoundingBox::from_point_cloud([first, second, max_bound]);
    assert_eq!(bbox.to_min_max(), (Vec3::new(1.0, 5.0, -10.0), max_bound));

    let min_bound = Vec3::new(-10.0, -10.0, -10.0);
    let bbox = BoundingBox::from_point_cloud([first, second, max_bound, min_bound]);
    assert_eq!(bbox.to_min_max(), (min_bound, max_bound));

    // Completely contained in box so no change.
    let extra = Vec3::new(3.0, -1.0, 3.0);
    let bbox = BoundingBox::from_point_cloud([first, second, max_bound, min_bound, extra]);
    assert_eq!(bbox.to_min_max(), (min_bound, max_bound));
}

#[test]
fn from_union() {
    let empty: [BoundingBox; 0] = [];
    assert!(BoundingBox::from_cloud(empty).is_empty());

    let start_max = Vec3::new(10.0, 9.0, 8.0);
    let start = BoundingBox::new(Vec3::new(1.0, 2.0, 3.0), start_max);
    assert_eq!(BoundingBox::from_cloud([start]), start);

    let disjoint_min = Vec3::new(-1.0, -2.0, -3.0);
    let disjoint = BoundingBox::new(disjoint_min, Vec3::new(0.0, 1.0, 2.0));
    let (min, max) = BoundingBox::from_cloud([start, disjoint]).to_min_max();
    assert_eq!((min, max), (disjoint_min, start_max));

    let contained = BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(3.0, 3.0, 3.0));
    let (min, max) = BoundingBox::from_cloud([start, disjoint, contained]).to_min_max();
    assert_eq!((min, max), (disjoint_min, start_max));

    let intersected = BoundingBox::new(Vec3::new(0.0, -100.0, 0.0), Vec3::new(3.0, 100.0, 3.0));
    let (min, max) =
        BoundingBox::from_cloud([start, disjoint, contained, intersected]).to_min_max();
    assert_eq!(min, Vec3::new(-1.0, -100.0, -3.0));
    assert_eq!(max, Vec3::new(10.0, 100.0, 8.0));
}

#[test]
fn containment() {
    let bbox = BoundingBox::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(5.0, 4.0, 3.0));

    assert!(bbox.contains_point(Vec3::ZERO));
    assert!(bbox.contains_point(Vec3::new(5.0, 4.0, 3.0)));
    assert!(bbox.contains_point(Vec3::new(4.0, 3.0, 2.0)));
    assert!(bbox.contains_point(Vec3::new(-0.5, -1.0, -1.0)));

    assert!(!bbox.contains_point(Vec3::new(6.0, 0.0, 0.0)));
    assert!(!bbox.contains_point(Vec3::new(-6.0, 0.0, 0.0)));
    assert!(!bbox.contains_point(Vec3::new(0.0, 6.0, 0.0)));
    assert!(!bbox.contains_point(Vec3::new(0.0, -6.0, 0.0)));
    assert!(!bbox.contains_point(Vec3::new(0.0, 0.0, 6.0)));
    assert!(!bbox.contains_point(Vec3::new(0.0, 0.0, -6.0)));

    assert!(!bbox.contains_bounds(&BoundingBox::empty()));
    assert!(!BoundingBox::empty().contains_bounds(&bbox));

    assert!(bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(-0.5, -1.0, -2.0),
        Vec3::new(1.0, 2.0, 3.0)
    )));
    assert!(!bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(-6.0, 0.0, 0.0),
        Vec3::new(0.1, 0.1, 0.1),
    )));
    assert!(!bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(0.0, -6.0, 0.0),
        Vec3::new(0.1, 0.1, 0.1),
    )));
    assert!(!bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, -6.0),
        Vec3::new(0.1, 0.1, 0.1),
    )));
    assert!(!bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(6.0, 0.1, 0.1),
    )));
    assert!(!bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.1, 6.0, 0.1),
    )));
    assert!(!bbox.contains_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.1, 0.1, 6.0),
    )));
}

#[test]
fn intersections() {
    let bbox = BoundingBox::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(5.0, 4.0, 3.0));

    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(-0.5, -1.0, -2.0),
        Vec3::new(1.0, 2.0, 3.0)
    )));
    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(-6.0, 0.0, 0.0),
        Vec3::new(0.1, 0.1, 0.1),
    )));
    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, -6.0, 0.0),
        Vec3::new(0.1, 0.1, 0.1),
    )));
    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, -6.0),
        Vec3::new(0.1, 0.1, 0.1),
    )));
    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(6.0, 0.1, 0.1),
    )));
    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.1, 6.0, 0.1),
    )));
    assert!(bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.1, 0.1, 6.0),
    )));

    assert!(!bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(5.9, 0.0, 0.0),
        Vec3::new(6.0, 6.0, 6.0),
    )));
    assert!(!bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, 5.9, 0.0),
        Vec3::new(6.0, 6.0, 6.0),
    )));
    assert!(!bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(0.0, 0.0, 5.9),
        Vec3::new(6.0, 6.0, 6.0),
    )));
    assert!(!bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(-6.0, -6.0, -6.0),
        Vec3::new(-5.9, 0.0, 0.0),
    )));
    assert!(!bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(-6.0, -6.0, -6.0),
        Vec3::new(0.0, -5.9, 0.0),
    )));
    assert!(!bbox.intersects_bounds(&BoundingBox::new(
        Vec3::new(-6.0, -6.0, -6.0),
        Vec3::new(0.0, 0.0, -5.9),
    )));
}

#[test]
fn transform() {
    assert_eq!(
        BoundingBox::empty().transform(&Transform::<XYZ>::new(Vec3::new(1.0, 2.0, 3.0), 0.75)),
        BoundingBox::empty()
    );

    let root_2 = 2.0f32.sqrt();
    let (actual_min, actual_max) =
        BoundingBox::new(Vec3::new(1.0, 3.0, 2.0), Vec3::new(6.0, 4.0, 5.0))
            .transform(&Transform::<XYZ>::new(
                Vec3::new(-4.0, 1.0, -3.0),
                PI * -0.75,
            ))
            .to_min_max();
    let expected_min = Vec3::new(-3.0 / root_2 - 4.0, -10.0 / root_2 + 1.0, -1.0);
    assert!(
        actual_min.abs_diff_eq(expected_min, 1e-6),
        "actual_min={actual_min} expected_min={expected_min}"
    );
    let expected_max = Vec3::new(3.0 / root_2 - 4.0, -4.0 / root_2 + 1.0, 2.0);
    assert!(
        actual_max.abs_diff_eq(expected_max, 1e-6),
        "actual_max={actual_max} expected_max={expected_max}"
    );
}

#[test]
fn octant_hierarchy() {
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
    .map(|(index, min, max)| (BoundingBox::new(min, max), index));

    let bbh = BoundingBoxHierarchy::new(&mut values);

    assert_eq!(bbh.depth(), 4);

    assert!(bbh.query(BoundingBox::empty()).is_empty());
    assert_eq!(
        bbh.query(BoundingBox::new(
            Vec3::new(1.0, 5.0, 1.0),
            Vec3::new(4.0, 8.0, 4.0),
        )),
        [2]
    );
    assert_eq!(
        bbh.query(BoundingBox::new(
            Vec3::new(1.0, 5.0, 1.0),
            Vec3::new(6.0, 8.0, 4.0),
        )),
        [2, 3]
    );
    assert_eq!(
        bbh.query(BoundingBox::new(
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(7.0, 7.0, 7.0),
        )),
        [0, 1, 2, 3, 4, 5, 6, 7]
    );
}

#[test]
fn hierarchy_with_same_big_dimension() {
    let mut values = [
        (0, Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 11.0)),
        (1, Vec3::new(4.0, 1.0, 1.0), Vec3::new(5.0, 2.0, 11.0)),
    ]
    .map(|(index, min, max)| (BoundingBox::new(min, max), index));

    let bbh = BoundingBoxHierarchy::new(&mut values);
    assert_eq!(
        bbh.query(BoundingBox::new(
            Vec3::new(1.5, 1.5, 1.5),
            Vec3::new(1.5, 1.5, 1.5)
        )),
        [0],
    );
}
