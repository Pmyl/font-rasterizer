use bitmap::Point;
use std::ops::{Add, Mul};

use crate::glyf::{GlyfData, GlyfDefinition};

pub fn rasterize_glyph_to_bitmap(glyph: &GlyfData) {
    let padding = 16;
    let variations_of_big: Vec<(usize, usize)> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (0, 2),
        (1, 2),
        (2, 2),
        (3, 2),
        (0, 3),
        (1, 3),
        (2, 3),
        (3, 3),
    ];

    let height = (glyph.y_max - glyph.y_min) as usize + padding;
    let width = (glyph.x_max - glyph.x_min) as usize + padding;
    let mut bitmap_maker = bitmap::BitmapMaker::new(width, height);

    match &glyph.definition {
        GlyfDefinition::Simple(simple_glyf_definition) => {
            bitmap_maker = draw_lines(
                glyph,
                padding,
                &variations_of_big,
                height,
                bitmap_maker,
                simple_glyf_definition,
            );
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

    let mut image_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("image.bmp")
        .expect("Should be able to open a file");

    bitmap.write(&mut image_file).unwrap();
}

fn draw_lines(
    glyph: &GlyfData,
    padding: usize,
    variations_of_big: &Vec<(usize, usize)>,
    height: usize,
    mut bitmap_maker: bitmap::BitmapMaker,
    simple_glyf_definition: &crate::glyf::SimpleGlyfDefinition,
) -> bitmap::BitmapMaker {
    for trio in simple_glyf_definition
        .x_coordinates
        .iter()
        .zip(&simple_glyf_definition.y_coordinates)
        .zip(&simple_glyf_definition.flags)
        .collect::<Vec<_>>()
        .chunks(3)
    {
        if trio.len() != 3 {
            continue;
        }

        if !trio[0].1.on_curve || trio[1].1.on_curve || !trio[2].1.on_curve {
            continue;
        }

        println!("FOUND A CURVE TO DRAW");

        let p0 = trio[0];
        let p1 = trio[1];
        let p2 = trio[2];

        let p0_to_p1_distance = distance(
            Point::new(*p0.0.0 as usize, *p0.0.1 as usize),
            Point::new(*p1.0.0 as usize, *p1.0.1 as usize),
        );
        let p1_to_p2_distance = distance(
            Point::new(*p1.0.0 as usize, *p1.0.1 as usize),
            Point::new(*p2.0.0 as usize, *p2.0.1 as usize),
        );
        let total_distance = p0_to_p1_distance + p1_to_p2_distance;

        let mut t: f32 = 0.0;
        let step: f32 = 1.0 / total_distance as f32;

        let p0 = PointF {
            x: *p0.0.0 as f32,
            y: *p0.0.1 as f32,
        };

        let p1 = PointF {
            x: *p1.0.0 as f32,
            y: *p1.0.1 as f32,
        };

        let p2 = PointF {
            x: *p2.0.0 as f32,
            y: *p2.0.1 as f32,
        };

        for _ in 0..total_distance {
            t += step;
            let point_on_curve = curve_point(p0, p1, p2, t);
            for variations in variations_of_big {
                bitmap_maker = bitmap_maker.with(
                    Point {
                        x: (point_on_curve.x as i16 - glyph.x_min) as usize
                            + padding / 2
                            + variations.0,
                        y: height
                            - ((point_on_curve.y as i16 - glyph.y_min) as usize
                                + padding / 2
                                + variations.1),
                    },
                    0x0000FF,
                );
            }
        }
    }

    bitmap_maker
}

fn draw_points(
    glyph: &GlyfData,
    padding: usize,
    variations_of_big: &Vec<(usize, usize)>,
    height: usize,
    mut bitmap_maker: bitmap::BitmapMaker,
    simple_glyf_definition: &crate::glyf::SimpleGlyfDefinition,
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

fn curve_point(p0: PointF, p1: PointF, p2: PointF, t: f32) -> PointF {
    (1.0 - t).powf(2.0) * p0 + 2.0 * t * (1.0 - t) * p1 + t.powf(2.0) * p2
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

fn distance(p0: Point, p1: Point) -> usize {
    (p0.x.abs_diff(p1.x).pow(2) + p0.y.abs_diff(p1.y).pow(2)).isqrt()
}
