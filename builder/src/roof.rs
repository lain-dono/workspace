use super::math::{remap, QuadCurve};
use crate::giz::{find_hover, screen_scale, Axis, InputState, Painter, X_COLOR, Y_COLOR, Z_COLOR};
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use bevy_egui::EguiContexts;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::f32::consts::{FRAC_PI_2, PI};

pub fn plugin(app: &mut App) {
    app.init_gizmo_group::<RoofGizmos>()
        .add_systems(Update, (ui_editor, draw_normals, update_houses).chain())
        .add_systems(Startup, (setup_roof_gismos, setup));
}

fn ui_editor(mut contexts: EguiContexts, mut query: Query<&mut HouseState>) {
    egui::Window::new("house states").show(contexts.ctx_mut(), |ui| {
        fn drag(
            ui: &mut egui::Ui,
            value: &mut f32,
            range: std::ops::RangeInclusive<f32>,
            speed: f32,
        ) {
            ui.add(egui::DragValue::new(value).range(range).speed(speed));
        }

        for mut house in &mut query {
            ui.label("depth/elevation:");
            ui.horizontal(|ui| {
                drag(ui, &mut house.depth, 0.0001..=1.0, 0.01);
                drag(ui, &mut house.elevation, 0.0001..=1.0, 0.01);
            });

            ui.label("y:");
            ui.horizontal(|ui| {
                let range = -1.0..=house.middle_y;
                drag(ui, &mut house.bottom_y, range, 0.1);
                let range = house.bottom_y..=house.roof_y;
                drag(ui, &mut house.middle_y, range, 0.1);
                let range = house.roof_y..=150.0;
                drag(ui, &mut house.roof_y, range, 0.1);
            });

            ui.label("roof:");
            ui.horizontal(|ui| {
                let range = house.wall_min.x..=house.roof_max.x;
                drag(ui, &mut house.roof_min.x, range, 0.1);
                let range = house.wall_min.y..=house.roof_max.y;
                drag(ui, &mut house.roof_min.y, range, 0.1);
            });
            ui.horizontal(|ui| {
                let range = house.roof_min.x..=house.wall_max.x;
                drag(ui, &mut house.roof_max.x, range, 0.1);
                let range = house.roof_min.y..=house.wall_max.y;
                drag(ui, &mut house.roof_max.y, range, 0.1);
            });

            ui.label("wall:");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut house.wall_min.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut house.wall_min.y).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut house.wall_max.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut house.wall_max.y).speed(0.1));
            });
        }
    });
}

#[derive(Component)]
pub struct HouseState {
    pub tile_size: [f32; 3],

    pub depth: f32,
    pub elevation: f32,

    pub roof_y: f32,
    pub middle_y: f32,
    pub bottom_y: f32,

    pub wall_min: Vec2,
    pub wall_max: Vec2,

    pub roof_min: Vec2,
    pub roof_max: Vec2,
}

impl Default for HouseState {
    fn default() -> Self {
        Self {
            tile_size: [0.218, 0.03, 0.317],

            depth: 0.4,
            elevation: 0.2,

            bottom_y: 0.0,
            middle_y: 2.2,
            roof_y: 4.2,

            wall_min: Vec2::new(-3.0, -3.0),
            wall_max: Vec2::new(2.5, 2.5),

            roof_min: Vec2::new(-1.0, -1.0),
            roof_max: Vec2::new(1.0, 1.0),
        }
    }
}

impl HouseState {
    fn generate(&self) -> (Vec<TileInstance>, MeshBuilder) {
        fn rotate(x: f32, y: f32) -> Quat {
            Quat::from_rotation_y(y) * Quat::from_rotation_x(x)
        }

        let a = f32::to_radians(-5.5);
        let b = f32::to_radians(3.5);
        let c = f32::to_radians(5.0);
        let post_rotation = |v: Vec3| -> Quat {
            let mut rng = rng_from_hash(v.to_array().map(f32::to_ne_bytes));
            let b = rng.gen_range(-b..=b);
            let c = rng.gen_range(-c..=c);
            Quat::from_euler(EulerRot::XYZ, a, b, c)
        };

        let [x_length, y_length, z_length] = self.tile_size;
        let half_size = Vec3::new(x_length, y_length, z_length) / 2.0;

        let mut builder_a = MeshBuilder::default();
        let mut builder_b = MeshBuilder::default();
        let mut builder_c = MeshBuilder::default();
        let mut builder_d = MeshBuilder::default();

        let depth_range = (self.wall_max.y, self.roof_max.y);
        let width_min = (self.wall_min.x, self.roof_min.x);
        let width_max = (self.wall_max.x, self.roof_max.x);
        let iter_a = self
            .single_roof_iter(depth_range, width_min, width_max)
            .map(|(angle, width, elevation, depth)| {
                let origin = Vec3::new(width, elevation, depth);
                let rotation = rotate(-angle - FRAC_PI_2, 0.0) * post_rotation(origin);
                let transform = Mat4::from_rotation_translation(rotation, origin);

                builder_a.cuboid(half_size, |p| {
                    let mut p = transform.transform_point3(p);
                    let min = remap(p.z, depth_range, width_min);
                    let max = remap(p.z, depth_range, width_max);
                    if min <= max {
                        p.x = p.x.clamp(min, max);
                    }
                    p
                });
                TileInstance { origin, rotation }
            });

        let depth_range = (self.wall_min.y, self.roof_min.y);
        let width_min = (self.wall_min.x, self.roof_min.x);
        let width_max = (self.wall_max.x, self.roof_max.x);
        let iter_b = self
            .single_roof_iter(depth_range, width_min, width_max)
            .map(|(angle, width, elevation, depth)| {
                let origin = Vec3::new(width, elevation, depth);
                let rotation = rotate(angle - FRAC_PI_2, PI) * post_rotation(origin);
                let transform = Mat4::from_rotation_translation(rotation, origin);
                builder_b.cuboid(half_size, |p| {
                    let mut p = transform.transform_point3(p);
                    let min = remap(p.z, depth_range, width_min);
                    let max = remap(p.z, depth_range, width_max);
                    if min <= max {
                        p.x = p.x.clamp(min, max);
                    }
                    p
                });
                TileInstance { origin, rotation }
            });

        let depth_range = (self.wall_min.x, self.roof_min.x);
        let width_min = (self.wall_min.y, self.roof_min.y);
        let width_max = (self.wall_max.y, self.roof_max.y);
        let iter_c = self
            .single_roof_iter(depth_range, width_min, width_max)
            .map(|(angle, width, elevation, depth)| {
                let origin = Vec3::new(depth, elevation, width);
                let rotation = rotate(angle - FRAC_PI_2, -FRAC_PI_2) * post_rotation(origin);
                let transform = Mat4::from_rotation_translation(rotation, origin);
                builder_c.cuboid(half_size, |p| {
                    let mut p = transform.transform_point3(p);
                    let min = remap(p.x, depth_range, width_min);
                    let max = remap(p.x, depth_range, width_max);
                    if min <= max {
                        p.z = p.z.clamp(min, max);
                    }
                    p
                });
                TileInstance { origin, rotation }
            });

        let depth_range = (self.wall_max.x, self.roof_max.x);
        let width_min = (self.wall_min.y, self.roof_min.y);
        let width_max = (self.wall_max.y, self.roof_max.y);
        let iter_d = self
            .single_roof_iter(depth_range, width_min, width_max)
            .map(|(angle, width, elevation, depth)| {
                let origin = Vec3::new(depth, elevation, width);
                let rotation = rotate(-angle - FRAC_PI_2, FRAC_PI_2) * post_rotation(origin);
                let transform = Mat4::from_rotation_translation(rotation, origin);
                builder_d.cuboid(half_size, |p| {
                    let mut p = transform.transform_point3(p);
                    let min = remap(p.x, depth_range, width_min);
                    let max = remap(p.x, depth_range, width_max);
                    if min <= max {
                        p.z = p.z.clamp(min, max);
                    }
                    p
                });
                TileInstance { origin, rotation }
            });

        let mut instances = vec![];
        instances.extend(iter_a);
        instances.extend(iter_b);
        instances.extend(iter_c);
        instances.extend(iter_d);

        let mut builder = MeshBuilder::default();
        builder.append(builder_a);
        builder.append(builder_b);
        builder.append(builder_c);
        builder.append(builder_d);

        // for instance in &instances {
        //     let transform = Mat4::from_rotation_translation(instance.rotation, instance.origin);
        //     builder.cuboid(half_size, |p| transform.transform_point3(p));
        // }

        (instances, builder)
    }

    fn single_roof_iter(
        &self,
        depth_range: (f32, f32),
        width_min: (f32, f32),
        width_max: (f32, f32),
    ) -> impl Iterator<Item = (f32, f32, f32, f32)> + '_ {
        self.roof_intervals(depth_range)
            .flat_map(move |(angle, depth, elevation)| {
                let w0 = remap(depth, depth_range, width_min);
                let w1 = remap(depth, depth_range, width_max);
                let stride = self.tile_size[0];
                let (start, count) = Self::start_count(w0, w1, stride);
                (0..=count).map(move |i| (angle, start + stride * i as f32, elevation, depth))
            })
    }

    fn start_count(w0: f32, w1: f32, stride: f32) -> (f32, usize) {
        let delta = (w1 - w0) / stride;
        let count = delta.round();
        let start = w0 + stride * (delta - count) / 2.0;
        (start, count as usize)
    }

    fn roof_intervals(&self, (x0, x2): (f32, f32)) -> impl Iterator<Item = (f32, f32, f32)> {
        let (y0, y2) = (self.middle_y, self.roof_y);

        let x1 = remap(self.depth, (0.0, 1.0), (x0, x2));
        let y1 = remap(self.elevation, (0.0, 1.0), (y0, y2));

        let p0 = Vec2::new(x0, y0);
        let p1 = Vec2::new(x1, y1);
        let p2 = Vec2::new(x2, y2);

        let intervals = QuadCurve { p0, p1, p2 }.intervals(self.tile_size[2]);
        intervals.map(|(_, p, n)| (n.to_angle(), p.x, p.y))
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let house = HouseState::default();

    let (transform, mesh) = {
        let x_length = house.wall_max.x - house.wall_min.x;
        let y_length = house.middle_y - house.bottom_y;
        let z_length = house.wall_max.y - house.wall_min.y;

        let x = x_length * 0.5 + house.wall_min.x;
        let y = y_length * 0.5 + house.bottom_y;
        let z = z_length * 0.5 + house.wall_min.y;

        let mesh = Cuboid::new(x_length, y_length, z_length);
        (Transform::from_xyz(x, y, z), meshes.add(mesh))
    };

    commands.spawn((
        transform,
        Mesh3d(mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.80, 0.83, 0.55),
            perceptual_roughness: 0.8,
            ..default()
        })),
    ));

    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.05, 0.24, 0.14),
            perceptual_roughness: 0.8,
            ..default()
        })),
        house,
        GizmoInstances::default(),
    ));
}

fn update_houses(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Mesh3d, &mut GizmoInstances, Ref<HouseState>)>,
) {
    for (entity, mut mesh, mut gizmo, house) in
        query.iter_mut().filter(|(_, _, _, h)| h.is_changed())
    {
        let (instances, builder) = house.generate();
        mesh.0 = meshes.add(builder.build(RenderAssetUsages::RENDER_WORLD));
        gizmo.instances = instances;

        commands
            .entity(entity)
            .remove::<bevy::render::primitives::Aabb>();
    }
}

fn rng_from_hash<V: std::hash::Hash>(value: V) -> SmallRng {
    use std::hash::{DefaultHasher, Hasher};
    let mut s = DefaultHasher::new();
    value.hash(&mut s);
    SmallRng::seed_from_u64(s.finish())
}

pub fn paint_house_gizmo(painter: &mut Painter, input: InputState, house: &mut HouseState) {
    let translation = Vec3::ZERO;
    let rotation = Quat::IDENTITY;
    let transform = Mat4::from_rotation_translation(rotation, translation);

    // let wireframe_stroke = (0.25, epaint::Color32::WHITE);
    let wireframe_stroke = (0.5, epaint::Color32::WHITE);
    let dash = 6.0;
    let gap = 6.0;

    let bottom_a = Vec3::new(house.wall_max.x, house.bottom_y, house.wall_max.y);
    let bottom_b = Vec3::new(house.wall_min.x, house.bottom_y, house.wall_max.y);
    let bottom_c = Vec3::new(house.wall_min.x, house.bottom_y, house.wall_min.y);
    let bottom_d = Vec3::new(house.wall_max.x, house.bottom_y, house.wall_min.y);

    let middle_a = Vec3::new(house.wall_max.x, house.middle_y, house.wall_max.y);
    let middle_b = Vec3::new(house.wall_min.x, house.middle_y, house.wall_max.y);
    let middle_c = Vec3::new(house.wall_min.x, house.middle_y, house.wall_min.y);
    let middle_d = Vec3::new(house.wall_max.x, house.middle_y, house.wall_min.y);

    let roof_a = Vec3::new(house.roof_max.x, house.roof_y, house.roof_max.y);
    let roof_b = Vec3::new(house.roof_min.x, house.roof_y, house.roof_max.y);
    let roof_c = Vec3::new(house.roof_min.x, house.roof_y, house.roof_min.y);
    let roof_d = Vec3::new(house.roof_max.x, house.roof_y, house.roof_min.y);

    painter.dashed_line(transform, bottom_a, bottom_b, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, bottom_b, bottom_c, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, bottom_c, bottom_d, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, bottom_d, bottom_a, wireframe_stroke, dash, gap);

    painter.dashed_line(transform, middle_a, middle_b, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, middle_b, middle_c, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, middle_c, middle_d, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, middle_d, middle_a, wireframe_stroke, dash, gap);

    painter.dashed_line(transform, roof_a, roof_b, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, roof_b, roof_c, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, roof_c, roof_d, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, roof_d, roof_a, wireframe_stroke, dash, gap);

    painter.dashed_line(transform, bottom_a, middle_a, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, bottom_b, middle_b, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, bottom_c, middle_c, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, bottom_d, middle_d, wireframe_stroke, dash, gap);

    painter.dashed_line(transform, middle_a, roof_a, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, middle_b, roof_b, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, middle_c, roof_c, wireframe_stroke, dash, gap);
    painter.dashed_line(transform, middle_d, roof_d, wireframe_stroke, dash, gap);

    let len = 40.0;
    let stroke_width = 8.0;
    let scale_length = screen_scale(&painter.viewport, transform);

    let mid_y = (house.middle_y + house.bottom_y) / 2.0;
    let roof_y = house.roof_y;

    let mid_nx = Vec3::new(house.wall_min.x, mid_y, 0.0);
    let mid_px = Vec3::new(house.wall_max.x, mid_y, 0.0);
    let mid_nz = Vec3::new(0.0, mid_y, house.wall_min.y);
    let mid_pz = Vec3::new(0.0, mid_y, house.wall_max.y);

    let roof_nx = Vec3::new(house.roof_min.x, roof_y, 0.0);
    let roof_px = Vec3::new(house.roof_max.x, roof_y, 0.0);
    let roof_nz = Vec3::new(0.0, roof_y, house.roof_min.y);
    let roof_pz = Vec3::new(0.0, roof_y, house.roof_max.y);

    let roof_py = Vec3::new(0.0, roof_y, 0.0);

    let wall_nx = Axis::arrow_nx(transform, scale_length, 0.0, len).origin(mid_nx);
    let wall_px = Axis::arrow_px(transform, scale_length, 0.0, len).origin(mid_px);
    let wall_nz = Axis::arrow_nz(transform, scale_length, 0.0, len).origin(mid_nz);
    let wall_pz = Axis::arrow_pz(transform, scale_length, 0.0, len).origin(mid_pz);

    let roof_nx = Axis::arrow_nx(transform, scale_length, 0.0, len).origin(roof_nx);
    let roof_px = Axis::arrow_px(transform, scale_length, 0.0, len).origin(roof_px);
    let roof_nz = Axis::arrow_nz(transform, scale_length, 0.0, len).origin(roof_nz);
    let roof_pz = Axis::arrow_pz(transform, scale_length, 0.0, len).origin(roof_pz);

    let roof_py = Axis::arrow_py(transform, scale_length, 0.0, len).origin(roof_py);

    let items = [
        (0, wall_nx, X_COLOR),
        (1, wall_px, X_COLOR),
        (2, wall_nz, Z_COLOR),
        (3, wall_pz, Z_COLOR),
        (4, roof_nx, X_COLOR),
        (5, roof_px, X_COLOR),
        (6, roof_nz, Z_COLOR),
        (7, roof_pz, Z_COLOR),
        (8, roof_py, Y_COLOR),
    ];

    let focus_distance = stroke_width * 0.5 + 5.0;
    let hover = find_hover(&painter.viewport, &input, focus_distance, items.iter());

    for (mode, axis, color) in items {
        // let is_active = gizmo.active == Some(mode);
        let is_hovered = hover == Some(mode);

        let color = color.linear_multiply(if is_hovered { 1.0 } else { 0.7 });
        axis.paint(painter, (stroke_width, color));
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct RoofGizmos;

fn setup_roof_gismos(mut configs: ResMut<GizmoConfigStore>) {
    let (config, _) = configs.config_mut::<RoofGizmos>();
    // config.depth_bias = -1.0;
}

fn draw_normals(mut gizmos: Gizmos<RoofGizmos>, query: Query<&GizmoInstances>) {
    return;
    for GizmoInstances { instances } in &query {
        for instance in instances {
            let normal = Dir3::new(instance.rotation * Vec3::Y).unwrap();
            let tangent = Dir3::new(instance.rotation * Vec3::Z).unwrap();

            let origin = instance.origin;
            let end_normal = instance.origin + normal * 0.04;
            let end_tangent = instance.origin + tangent * 0.24;

            gizmos.line(origin, end_normal, crate::giz::_X_COLOR);
            gizmos.line(origin, end_tangent, crate::giz::_Z_COLOR);
        }
    }
}

#[derive(Clone)]
pub struct TileInstance {
    pub origin: Vec3,
    pub rotation: Quat,
}

#[derive(Component, Default)]
pub struct GizmoInstances {
    pub instances: Vec<TileInstance>,
}

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn build(self, asset_usage: RenderAssetUsages) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, asset_usage)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_indices(Indices::U32(self.indices))
    }

    fn cuboid(&mut self, half_size: Vec3, pmap: impl Fn(Vec3) -> Vec3) {
        let min = -half_size;
        let max = half_size;

        // Suppose Y-up right hand, and camera look from +Z to -Z

        let p000 = pmap(Vec3::new(min.x, min.y, min.z));
        let p001 = pmap(Vec3::new(min.x, min.y, max.z));

        let p010 = pmap(Vec3::new(min.x, max.y, min.z));
        let p011 = pmap(Vec3::new(min.x, max.y, max.z));

        let p100 = pmap(Vec3::new(max.x, min.y, min.z));
        let p101 = pmap(Vec3::new(max.x, min.y, max.z));

        let p110 = pmap(Vec3::new(max.x, max.y, min.z));
        let p111 = pmap(Vec3::new(max.x, max.y, max.z));

        let uv_a = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
        let uv_b = [[1., 0.], [0., 0.], [0., 1.], [1., 1.]];

        self.quad([p100, p110, p111, p101], uv_a); // Right
        self.quad([p110, p010, p011, p111], uv_b); // Top
        self.quad([p001, p101, p111, p011], uv_a); // Front
        self.quad([p001, p011, p010, p000], uv_b); // Left

        // self.quad([p101, p001, p000, p100], vmap(Vec3::NEG_Y), uv_a); // Bottom *
        // self.quad([p010, p110, p100, p000], vmap(Vec3::NEG_Z), uv_b); // Back *
    }

    fn quad(&mut self, p: [Vec3; 4], uv: [[f32; 2]; 4]) {
        let na = face_normal(p[0], p[1], p[2]);
        let nb = face_normal(p[2], p[3], p[0]);

        let p = p.map(|p| p.to_array());

        if na.abs_diff_eq(nb, 0.001) {
            let nc = (na + nb).normalize();

            let start = self.positions.len() as u32;
            self.indices.extend([0, 1, 2, 2, 3, 0].map(|i| start + i));
            self.positions.extend(p);
            self.normals.extend([na, nc, nc, nb].map(|p| p.to_array()));
            self.uvs.extend(uv);
        } else {
            let start = self.positions.len() as u32;
            self.indices.extend([0, 1, 2].map(|i| start + i));
            self.positions.extend([p[0], p[1], p[2]]);
            self.normals.extend([na, na, na].map(|p| p.to_array()));
            self.uvs.extend([uv[0], uv[1], uv[2]]);

            let start = self.positions.len() as u32;
            self.indices.extend([0, 1, 2].map(|i| start + i));
            self.positions.extend([p[2], p[3], p[0]]);
            self.normals.extend([nb, nb, nb].map(|p| p.to_array()));
            self.uvs.extend([uv[2], uv[3], uv[0]]);
        }
    }

    fn append(&mut self, other: Self) {
        let base_index = self.positions.len() as u32;

        let indices = other.indices.into_iter();
        self.indices.extend(indices.map(|index| base_index + index));

        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.uvs.extend(other.uvs);
    }
}

fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize()
}
