use super::{Path, Shape, Vector};
use eframe::egui::{self, Color32, Pos2, Rgba};
use std::f32::consts::{FRAC_PI_6, PI};

pub struct Camera {
    pub angle: f32,
    pub scale: f32,
    pub x: f32,
    pub y: f32,
}

impl Camera {
    /// Precalculates transformation values based on the current angle and scale which in theory reduces costly cos and sin calls
    pub fn transformation(&self) -> [[f32; 2]; 2] {
        let &Self { angle, scale, .. } = self;
        [Self::tx(scale, angle), Self::tx(scale, PI - angle)]
    }

    fn tx(scale: f32, angle: f32) -> [f32; 2] {
        [scale * angle.cos(), scale * angle.sin()]
    }
}

pub struct Canvas {
    pub camera: Camera,
    pub light_direction: Vector,
    pub light_color: Color32,

    // The maximum color difference from shading
    pub color_difference: f32,

    pub transformation: [[f32; 2]; 2],

    pub data: Vec<egui::Shape>,
}

impl Canvas {
    pub fn new(x: f32, y: f32) -> Self {
        let camera = Camera {
            angle: FRAC_PI_6,
            scale: 50.0,
            x,
            y,
        };

        let light = Vector::new(2.0, -1.0, 3.0);

        Self {
            transformation: camera.transformation(),
            camera,

            light_direction: light.normalize(),
            light_color: Color32::WHITE,
            color_difference: 0.20,

            data: vec![],
        }
    }

    /// Sets the light position for drawing.
    pub fn set_light_direction(&mut self, pos: Vector) {
        self.light_direction = pos.normalize();
    }

    /// Adds a shape to the scene
    pub fn add_shape(&mut self, mut shape: Shape, color: Color32) {
        // Sort the list of faces by distance then map the entries, returning
        // only the path and not the added "further point" from earlier.
        shape.paths.sort_by(|a, b| b.depth().total_cmp(&a.depth()));

        for path in shape.paths {
            self.add_path(path, color);
        }
    }

    /// Adds a path to the scene
    pub fn add_path(&mut self, path: Path, color: Color32) {
        let fill = self.shade(path.points[..3].try_into().unwrap(), color);
        let stroke = (0.0, Color32::TRANSPARENT);
        let points = path.points.iter().map(|&p| self.map_point(p)).collect();
        let shape = egui::Shape::convex_polygon(points, fill, stroke);
        self.data.push(shape);
    }

    fn map_point(&self, point: Vector) -> Pos2 {
        // X rides along the angle extended from the origin
        // Y rides perpendicular to this angle (in isometric view: PI - angle)
        // Z affects the y coordinate of the drawn point
        let [mx, my] = self.transformation;
        let [ox, oy] = [self.camera.x, self.camera.y];

        let px = [point.x * mx[0], point.x * mx[1]];
        let py = [point.y * my[0], point.y * my[1]];
        let pz = point.z * self.camera.scale;

        Pos2::new(ox + px[0] + py[0], oy - px[1] - py[1] - pz)
    }

    fn shade(&self, points: [Vector; 3], color: Color32) -> Color32 {
        // Compute color
        let a = points[0] - points[1];
        let b = points[1] - points[2];

        let normal = a.cross(b).normalize();

        // Brightness is between -1 and 1 and is computed based
        // on the dot product between the light source vector and normal.
        let brightness = normal.dot(self.light_direction);
        lighten(color, brightness * self.color_difference, self.light_color)
    }
}

fn lighten(color: Color32, percentage: f32, light: Color32) -> Color32 {
    let color = Rgba::from(color);
    let light = Rgba::from(light);

    let [r, g, b] = [
        color[0] * light[0],
        color[1] * light[1],
        color[2] * light[2],
    ];

    let [h, s, l] = load_hsl([r, g, b]);
    let [r, g, b] = load_rgb([h, s, (l + percentage).min(1.0)]);

    Rgba::from_rgba_premultiplied(r, g, b, color[3]).into()
}

// Loads HSL values using the current RGB values
// Converted from:
// http://axonflux.com/handy-rgb-to-hsl-and-rgb-to-hsv-color-model-c
fn load_hsl([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    let l = (max + min) / 2.0;

    if max == min {
        // achromatic
        [0.0, 0.0, l]
    } else {
        let d = max - min;
        let s = d / if l > 0.5 { 2.0 - max - min } else { max + min };
        let h = match 0 {
            _ if r == max => (g - b) / d + (if g < b { 6.0 } else { 0.0 }),
            _ if g == max => (b - r) / d + 2.0,
            _ => (r - g) / d + 4.0,
        } / 6.0;
        [h, s, l]
    }
}

// Reloads RGB using HSL values
// Converted from:
// http://axonflux.com/handy-rgb-to-hsl-and-rgb-to-hsv-color-model-c
fn load_rgb([h, s, l]: [f32; 3]) -> [f32; 3] {
    if s == 0.0 {
        [l; 3] // achromatic
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let r = hue2rgb(p, q, h + 1.0 / 3.0);
        let g = hue2rgb(p, q, h);
        let b = hue2rgb(p, q, h - 1.0 / 3.0);
        [r, g, b]
    }
}

// Helper function to convert hue to rgb
// Taken from:
// http://axonflux.com/handy-rgb-to-hsl-and-rgb-to-hsv-color-model-c
fn hue2rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
