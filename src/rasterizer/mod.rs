use bitmap::Point;
use std::{
    fs::create_dir_all,
    ops::{Add, Mul, Sub},
    path::Path,
};

use crate::{
    font::{
        self,
        glyf::{GlyfData, GlyfDefinition, GlyfFlag},
    },
    rasterizer::pixel_map::{PixelInfo, PixelMap},
};

mod pixel_map;

pub fn rasterize_glyph_to_bitmap(glyph: &GlyfData, file_path: &Path) {
    let padding = 16;
    let variations_of_big: Vec<(usize, usize)> = vec![
        (0, 0),
        // (1, 0),
        // (2, 0),
        // (3, 0),
        // (0, 1),
        // (1, 1),
        // (2, 1),
        // (3, 1),
        // (0, 2),
        // (1, 2),
        // (2, 2),
        // (3, 2),
        // (0, 3),
        // (1, 3),
        // (2, 3),
        // (3, 3),
    ];

    let height = (glyph.y_max - glyph.y_min) as usize + padding;
    let width = (glyph.x_max - glyph.x_min) as usize + padding;
    let mut bitmap_maker = bitmap::BitmapMaker::new(width, height);
    let mut pixel_map = PixelMap::new(width, height);

    match &glyph.definition {
        GlyfDefinition::Simple(simple_glyf_definition) => {
            let mut start = 0;

            for contour in &simple_glyf_definition.end_pts_of_contours {
                let end = *contour as usize;
                draw_contour(
                    glyph,
                    padding,
                    &mut pixel_map,
                    &simple_glyf_definition.x_coordinates[start..=end],
                    &simple_glyf_definition.y_coordinates[start..=end],
                    &simple_glyf_definition.flags[start..=end],
                );
                start = end + 1;
            }

            bitmap_maker = fill_glyph(width, height, pixel_map, bitmap_maker);

            bitmap_maker = draw_points(
                glyph,
                padding,
                &variations_of_big,
                height,
                bitmap_maker,
                simple_glyf_definition,
            );
        }
        GlyfDefinition::Compound => {}
    }

    let bitmap = bitmap_maker.make().unwrap();

    create_dir_all(file_path.parent().unwrap()).unwrap();

    let mut image_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)
        .expect("Should be able to open a file");

    bitmap.write(&mut image_file).unwrap();
}

#[derive(Clone)]
struct ContourPoint {
    x: i16,
    y: i16,
    on_curve: bool,
}

fn draw_contour(
    glyph: &GlyfData,
    padding: usize,
    pixel_map: &mut PixelMap,
    xs: &[i16],
    ys: &[i16],
    flags: &[GlyfFlag],
) {
    let mut i = 0;
    let mut vertices = vec![];

    let get_p = |index: usize| -> Option<ContourPoint> {
        if xs.len() == index {
            Some(ContourPoint {
                x: *xs.get(0)?,
                y: *ys.get(0)?,
                on_curve: flags.get(0).map(|f| f.on_curve)?,
            })
        } else {
            Some(ContourPoint {
                x: *xs.get(index)?,
                y: *ys.get(index)?,
                on_curve: flags.get(index).map(|f| f.on_curve)?,
            })
        }
    };

    let mut virtual_p0 = None;
    loop {
        let Some(p0) = virtual_p0.take().or_else(|| get_p(i)) else {
            break;
        };

        vertices.push(p0.clone());

        i += 1;
        let Some(p1) = get_p(i) else {
            break;
        };

        if p1.on_curve {
            draw_straight_line(glyph, padding, pixel_map, (p0.x, p0.y), (p1.x, p1.y));
        } else {
            i += 1;
            let Some(p2) = get_p(i) else {
                break;
            };

            if !p2.on_curve {
                i -= 1;

                // b + (a-b)/ 2
                let vp2 = add_points(
                    (p2.x, p2.y),
                    divide_point(sub_points((p1.x, p1.y), (p2.x, p2.y)), 2),
                );

                draw_curve(
                    glyph,
                    padding,
                    pixel_map,
                    (p0.x, p0.y),
                    (p1.x, p1.y),
                    (vp2.0, vp2.1),
                );

                virtual_p0 = Some(ContourPoint {
                    x: vp2.0,
                    y: vp2.1,
                    on_curve: true,
                });
            } else {
                draw_curve(
                    glyph,
                    padding,
                    pixel_map,
                    (p0.x, p0.y),
                    (p1.x, p1.y),
                    (p2.x, p2.y),
                );
            }
        }
    }

    vertices.push(vertices[1].clone());
    for window in vertices.array_windows::<3>() {
        let prev = &window[0];
        let current = &window[1];
        let next = &window[2];

        if (prev.y > current.y && current.y < next.y)
            || (prev.y < current.y && current.y > next.y)
            || (next.y == current.y && current.y < prev.y)
            || (prev.y == current.y && current.y < next.y)
        {
            let x = (current.x - glyph.x_min) as usize + padding / 2;
            let y = (current.y - glyph.y_min) as usize + padding / 2;

            pixel_map.set(PixelInfo::InvisibleVertex, x, y);
        }

        if next.y == current.y && current.y > prev.y {
            let x = (current.x - glyph.x_min) as usize + padding / 2;
            let y = (current.y - glyph.y_min) as usize + padding / 2;

            pixel_map.set(PixelInfo::VisibleVertexZero, x, y);
        }

        if prev.y == current.y && current.y > next.y {
            let x = (current.x - glyph.x_min) as usize + padding / 2;
            let y = (current.y - glyph.y_min) as usize + padding / 2;

            pixel_map.set(PixelInfo::VisibleVertexOne, x, y);
        }
    }
}

fn sub_points(a: (i16, i16), b: (i16, i16)) -> (i16, i16) {
    (a.0 - b.0, a.1 - b.1)
}

fn add_points(a: (i16, i16), b: (i16, i16)) -> (i16, i16) {
    (a.0 + b.0, a.1 + b.1)
}

fn divide_point(a: (i16, i16), scalar: i16) -> (i16, i16) {
    (a.0 / scalar, a.1 / scalar)
}

fn draw_straight_line(
    glyph: &GlyfData,
    padding: usize,
    pixel_map: &mut PixelMap,
    p0: (i16, i16),
    p1: (i16, i16),
) {
    let total_distance = distance((p0.0, p0.1), (p1.0, p1.1));
    let pixel_info = if p0.1 > p1.1 {
        PixelInfo::Zero
    } else {
        PixelInfo::One
    };

    let mut t: f32 = 0.0;
    let step: f32 = 1.0 / total_distance as f32;

    let p0 = PointF {
        x: p0.0 as f32,
        y: p0.1 as f32,
    };

    let p1 = PointF {
        x: p1.0 as f32,
        y: p1.1 as f32,
    };

    let direction = p0 - p1;

    for _ in 0..total_distance {
        t += step;
        let point_on_line = line_point(p0, direction, t);

        let x = (point_on_line.x.round() as i16 - glyph.x_min) as usize + padding / 2;
        let y = (point_on_line.y.round() as i16 - glyph.y_min) as usize + padding / 2;

        pixel_map.set(pixel_info, x, y);
    }
}

fn draw_curve(
    glyph: &GlyfData,
    padding: usize,
    pixel_map: &mut PixelMap,
    p0: (i16, i16),
    p1: (i16, i16),
    p2: (i16, i16),
) {
    let p0_to_p1_distance = distance((p0.0, p0.1), (p1.0, p1.1));
    let p1_to_p2_distance = distance((p1.0, p1.1), (p2.0, p2.1));
    let total_distance = (p0_to_p1_distance + p1_to_p2_distance) * 10;

    let pixel_info = if p0.1 > p2.1 {
        PixelInfo::Zero
    } else {
        PixelInfo::One
    };

    let mut t: f32 = 0.0;
    let step: f32 = 1.0 / total_distance as f32;

    let p0 = PointF {
        x: p0.0 as f32,
        y: p0.1 as f32,
    };

    let p1 = PointF {
        x: p1.0 as f32,
        y: p1.1 as f32,
    };

    let p2 = PointF {
        x: p2.0 as f32,
        y: p2.1 as f32,
    };

    for _ in 0..total_distance {
        t += step;
        let point_on_curve = curve_point(p0, p1, p2, t);

        let x = (point_on_curve.x.round() as i16 - glyph.x_min) as usize + padding / 2;
        let y = (point_on_curve.y.round() as i16 - glyph.y_min) as usize + padding / 2;

        pixel_map.set(pixel_info, x, y);
    }
}

fn draw_points(
    glyph: &GlyfData,
    padding: usize,
    variations_of_big: &Vec<(usize, usize)>,
    height: usize,
    mut bitmap_maker: bitmap::BitmapMaker,
    simple_glyf_definition: &font::glyf::SimpleGlyfDefinition,
) -> bitmap::BitmapMaker {
    for ((x, y), flag) in simple_glyf_definition
        .x_coordinates
        .iter()
        .zip(&simple_glyf_definition.y_coordinates)
        .zip(&simple_glyf_definition.flags)
    {
        let colour = if flag.on_curve { 0x000000 } else { 0x00FF00 };
        for variations in variations_of_big {
            bitmap_maker = bitmap_maker.with(
                Point {
                    x: (x - glyph.x_min) as usize + padding / 2 + variations.0,
                    y: height - ((y - glyph.y_min) as usize + padding / 2 + variations.1),
                },
                colour,
            );
        }
    }

    bitmap_maker
}

fn fill_glyph(
    width: usize,
    height: usize,
    pixel_map: PixelMap,
    mut bitmap_maker: bitmap::BitmapMaker,
) -> bitmap::BitmapMaker {
    for x in 0..width {
        for y in 0..height {
            let PixelInfo::Empty = pixel_map.get_unchecked(x, y) else {
                // Contour
                bitmap_maker = bitmap_maker.with(Point::new(x, height - y), 0x44FF0000);
                continue;
            };

            let mut x0 = x;
            let mut crossing_count = 0;
            let mut last_count = 0;
            let mut vertex_touched = false;

            loop {
                x0 += 1;

                if x0 == width {
                    break;
                }

                match pixel_map.get_unchecked(x0, y) {
                    PixelInfo::Empty => {
                        last_count = 0;
                        vertex_touched = false;
                    }
                    PixelInfo::Zero if last_count != 1 && vertex_touched == false => {
                        crossing_count += 1;
                        last_count = 1;
                    }
                    PixelInfo::One if last_count != -1 && vertex_touched == false => {
                        crossing_count -= 1;
                        last_count = -1;
                    }
                    PixelInfo::Zero => {}
                    PixelInfo::One => {}
                    PixelInfo::InvisibleVertex => {
                        crossing_count -= last_count;
                        last_count = 0;
                        vertex_touched = true;
                    }
                    PixelInfo::VisibleVertexZero => {
                        crossing_count += 1;
                        vertex_touched = true;
                    }
                    PixelInfo::VisibleVertexOne => {
                        crossing_count -= 1;
                        vertex_touched = true;
                    }
                }
            }

            if crossing_count != 0 {
                // Inside
                bitmap_maker = bitmap_maker.with(Point::new(x, height - y), 0xFF440000);
            }
        }
    }

    bitmap_maker
}

fn curve_point(p0: PointF, p1: PointF, p2: PointF, t: f32) -> PointF {
    (1.0 - t).powf(2.0) * p0 + 2.0 * t * (1.0 - t) * p1 + t.powf(2.0) * p2
}

// t goes from 0 to 1
fn line_point(p0: PointF, direction: PointF, t: f32) -> PointF {
    p0 + t * direction
}

#[derive(Clone, Copy)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}

impl Mul<PointF> for f32 {
    type Output = PointF;

    fn mul(self, rhs: PointF) -> Self::Output {
        PointF {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

impl Add<PointF> for PointF {
    type Output = PointF;

    fn add(self, rhs: PointF) -> Self::Output {
        PointF {
            x: rhs.x + self.x,
            y: rhs.y + self.y,
        }
    }
}

impl Sub<PointF> for PointF {
    type Output = PointF;

    fn sub(self, rhs: PointF) -> Self::Output {
        PointF {
            x: rhs.x - self.x,
            y: rhs.y - self.y,
        }
    }
}

fn distance(p0: (i16, i16), p1: (i16, i16)) -> u16 {
    ((p0.0.abs_diff(p1.0) as usize).pow(2) + (p0.1.abs_diff(p1.1) as usize).pow(2)).isqrt() as u16
}
