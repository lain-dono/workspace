#![allow(clippy::identity_op, clippy::erasing_op)]

use image::GenericImage;
use palette::{
    FromColor, Hsv, OklabHue, Srgb, okhsl::Okhsl, okhsv::Okhsv, okhwb::Okhwb, oklch::Oklch,
};

struct Image {
    dst: image::RgbImage,
}

struct P(u32, u32);

impl std::ops::Add for P {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl std::ops::Mul for P {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl std::ops::Mul<u32> for P {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

fn main() {
    let path = "palette_.png";

    let mut dst = Image {
        // dst: image::RgbImage::new(256, 256),
        dst: image::RgbImage::new(128, 128),
    };

    //let m = image.sub_image(1, 7, 48, 48);

    // hsv_map(&mut dst, [1, 7], 8, 6);

    // okhsv_map(&mut dst, [51, 7], 4, [4, 9]);
    // okhsv_map(&mut dst, [51 + 16 + 1, 7], 8, [4, 9]);
    //

    dst.table(P(1, 1) + P(0, 0), 16, [6, 6], Okhsv::new);
    // dst.table(P(2, 1) + P(6, 0) * 16, 8, [6, 6], Okhsv::new);
    // dst.table(P(3, 1) + P(6, 0) * 24, 8, [6, 6], Okhsv::new);

    // dst.table([3 * 35 + 1, 1], 8, [4, 6], Okhsv::new);
    //dst.table([1 * 35 + 1, 1], 8, [4, 6], Okhsl::new);
    //dst.table([2 * 35 + 1, 1], 8, [4, 6], Hsv::new);

    //table(&mut dst, [0 * 35 + 1, 126 - 48], nxy, Okhsv::new);
    //table(&mut dst, [1 * 35 + 1, 126 - 48], nxy, Okhsv::new);

    //  for i in 0..3 {
    //      fill_hsv(&mut dst, [i * 35 + 1, 1], 8, [4, 6], okhsv_conv);
    //  }
    //
    //  for i in 0..3 {
    //      fill_hsv(&mut dst, [i * 35 + 1, 126 - 48], 8, [4, 6], oklch_conv);
    //  }

    //hsv_map(&mut dst, [1, 103], 4, 6);

    // _map(&mut dst, [51, 7], 8, 6);

    /*
    //dst.put_pixel(x, y, pixel);
    circle(&mut dst, 30, 30, 15, image::Rgb([0xFF, 0xFF, 0xFF]));
    // jesko_circle(&mut dst, 30, 30, 15, image::Rgb([0xFF, 0x00, 0xFF]));
    jesko_circle(&mut dst, 70, 70, 15, image::Rgb([0xFF, 0x00, 0xFF]));
    */

    match dst.dst.save(path) {
        Ok(()) => println!("see '{path}' for the result"),
        Err(e) => println!("failed to write '{path}': {e}"),
    }
}

fn hsv_conv(h: f32, s: f32, v: f32) -> [u8; 3] {
    let color = Hsv::new(h, s, v);
    let a = Srgb::from_color(color);
    let b = a.into_format();
    let c = b.into();
    c
}

fn conv<C>(color: impl Fn(f32, f32, f32) -> C) -> impl Fn(f32, f32, f32) -> [u8; 3]
where
    Srgb: FromColor<C>,
{
    move |h, s, v| Srgb::from_color(color(h, s, v)).into_format().into()
}

impl Image {
    fn table<C>(
        &mut self,
        xy: P,
        n: u32,
        [mx, my]: [u32; 2],
        color: impl Fn(f32, f32, f32) -> C + Copy,
    ) where
        Srgb: FromColor<C>,
    {
        let xy = [xy.0, xy.1];
        let mm = 360.0 / (mx * my) as f32;
        let nf = n as f32;
        self.fill(xy, [n * mx, n * my], move |x, y| {
            let h = (-mm * (y / n * mx + x / n) as f32).rem_euclid(360.0);
            let s = 1.0 - (x % n) as f32 / nf;
            let v = 1.0 - (y % n) as f32 / nf;
            conv(color)(h, s, v).into()
        });
    }

    fn fill(
        &mut self,
        [x, y]: [u32; 2],
        [width, height]: [u32; 2],
        fill: impl Fn(u32, u32) -> image::Rgb<u8>,
    ) {
        let image = image::RgbImage::from_fn(width, height, fill);
        self.dst.copy_from(&image, x, y).unwrap();
    }
}

fn remap(value: f32, from: [f32; 2], to: [f32; 2]) -> f32 {
    to[0] + (value - from[0]) * (to[1] - to[0]) / (from[1] - from[0])
}

impl Image {
    fn jesko_circle(&mut self, dx: u32, dy: u32, r: isize, pixel: image::Rgb<u8>) {
        let mut sub = SubI::new(&mut self.dst, dx, dy);

        let (mut x, mut y, mut t1) = (r, 0, r / 16);
        while x >= y {
            //Pixel (x, y) and all symmetric pixels are colored (8 times)

            for (j, k) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                // draw to all quadrants
                sub.set((j * x + r) as u32, (k * y + r) as u32, pixel);
                sub.set((k * y + r) as u32, (j * x + r) as u32, pixel);

                // set(j * x, k * y);
                // set(k * y, j * x);
            }

            y += 1;
            t1 += y;
            let t2 = t1 - x;
            if t2 >= 0 {
                t1 = t2;
                x -= 1;
            }
        }
    }

    fn circle(&mut self, dx: u32, dy: u32, r: isize, pixel: image::Rgb<u8>) {
        // image of size (2r + 1) x (2r + 1) to fit the circle
        let size = 2 * r as u32 + 1;

        let mut sub = SubI::new(&mut self.dst, dx, dy);

        let (mut x, mut y, mut p) = (r, 0, 1 - r); // initial values x0 = r, y0 = 0, p0 = 1 - r
        while x >= y {
            // while the point (x, y) is in the first octant
            for (j, k) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                // draw to all quadrants
                sub.set((j * x + r) as u32, (k * y + r) as u32, pixel);
                sub.set((k * y + r) as u32, (j * x + r) as u32, pixel);
            }

            (x, y) = (x - (p > 0) as isize, y + 1); // update x and y according to the radius error p
            p += if p > 0 { 1 - 2 * x + 2 * y } else { 1 + 2 * y }; // update the radius error p with updated x and y
        }

        //dst.copy_from(&image, dx, dy).unwrap();
    }
}

struct SubI<'a> {
    dst: &'a mut image::RgbImage,
    dx: u32,
    dy: u32,
}

impl<'a> SubI<'a> {
    fn new(dst: &'a mut image::RgbImage, dx: u32, dy: u32) -> Self {
        Self { dst, dx, dy }
    }

    fn set(&mut self, px: u32, py: u32, pixel: image::Rgb<u8>) {
        let [x, y] = [self.dx + px, self.dy + py];
        if let Some(p) = self.dst.get_pixel_mut_checked(x, y) {
            *p = pixel
        }
    }
}
