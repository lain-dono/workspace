use super::runtime::AnimaState;
use super::GridViewport;
use egui::*;

pub fn paint_bones(
    ui: &mut Ui,
    viewport: GridViewport,
    state: &AnimaState,
    shapes: &mut Vec<Shape>,
    selected: Option<usize>,
) -> Option<usize> {
    let px = ui.ctx().pixels_per_point().recip();
    let mut hovered = None;

    for (index, (bone, &transform)) in state.bones.iter().zip(&state.local_to_screen).enumerate() {
        let [r, g, b, a] = bone.color.0.to_be_bytes();
        let color = Color32::from_rgba_unmultiplied(r, g, b, a);
        let length = bone.length;

        let extra = vec2(length * 0.2, length * 0.1);
        let outer_radius = (viewport.zoom * length * 0.075).max(2.0);
        let inner_radius = outer_radius * 0.8;

        let a = transform.apply(0.0, 0.0).into();
        let b = transform.apply(extra.x, -extra.y).into();
        let c = transform.apply(length, 0.0).into();
        let d = transform.apply(extra.x, extra.y).into();

        let points = vec![a, b, c, d];

        let distance = Pos2::distance(b, d);
        let is_hovered = check_distance([a, c], viewport.pointer, distance);
        let is_selected = Some(index) == selected;

        if is_hovered {
            hovered = Some(index);
        }

        let stroke = if is_selected || is_hovered {
            Stroke::new(px, Color32::WHITE)
        } else {
            Stroke::default()
        };

        let path = epaint::PathShape::convex_polygon(points, color, stroke);
        shapes.push(Shape::Path(path));
        shapes.push(Shape::circle_filled(a, outer_radius, color));
        shapes.push(Shape::circle_filled(a, inner_radius, Color32::BLACK));

        if let Some(parent) = state.bones.get(bone.parent as usize) {
            let stroke = Stroke::new(px, Color32::RED);

            let parent_transform = state.local_to_screen[bone.parent as usize];
            let src: Pos2 = parent_transform.apply(parent.length / 2.0, 0.0).into();
            let dst: Pos2 = transform.apply(length / 2.0, 0.0).into();
            parent_arrow(shapes, src, dst, stroke, viewport.zoom);
        }
    }

    hovered
}

fn check_distance([a, b]: [Pos2; 2], pointer: Option<Pos2>, distance: f32) -> bool {
    if let Some(pointer) = pointer {
        let diff = b - a;
        let len_sq = diff.length_sq();
        if len_sq <= 0.0 {
            pointer.distance(a) < distance
        } else {
            let t = Vec2::dot(pointer - a, diff) / len_sq;
            let projection = a + diff * t.clamp(0.0, 1.0);
            pointer.distance(projection) < distance
        }
    } else {
        false
    }
}

fn parent_arrow(shapes: &mut Vec<Shape>, src: Pos2, dst: Pos2, stroke: Stroke, zoom: f32) {
    use egui::emath::*;
    let rot = Rot2::from_angle(std::f32::consts::TAU / 10.0);
    let len = 8.0 * zoom;
    let dir = (dst - src).normalized();
    let (dash, gap) = (4.0, 2.0);
    shapes.extend(Shape::dashed_line(&[src, dst], stroke, dash, gap));
    shapes.push(Shape::LineSegment {
        points: [dst, dst - len * (rot * dir)],
        stroke,
    });
    shapes.push(Shape::LineSegment {
        points: [dst, dst - len * (rot.inverse() * dir)],
        stroke,
    });
}
