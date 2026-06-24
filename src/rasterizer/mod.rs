use bitmap::Point;
use std::ops::{Add, Mul, Sub};

use crate::{
    font,
    font::glyf::{GlyfData, GlyfDefinition, GlyfFlag},
};

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
            let mut start = 0;

            for countor in &simple_glyf_definition.end_pts_of_contours {
                let end = countor;
                bitmap_maker = draw_lines(
                    glyph,
                    padding,
                    &variations_of_big,
                    height,
                    bitmap_maker,
                    simple_glyf_definition,
                    start,
                    *end as usize,
                );
                start = *end as usize + 1;
            }

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
    simple_glyf_definition: &font::glyf::SimpleGlyfDefinition,
    start: usize,
    end: usize,
) -> bitmap::BitmapMaker {
    let xs = &simple_glyf_definition.x_coordinates[start..=end];
    let ys = &simple_glyf_definition.y_coordinates[start..=end];
    let flags = &simple_glyf_definition.flags[start..=end];

    let mut i = 0;

    let get_p = |index: usize| -> Option<(i16, i16, &GlyfFlag)> {
        if xs.len() == index {
            Some((*xs.get(0)?, *ys.get(0)?, flags.get(0)?))
        } else {
            Some((*xs.get(index)?, *ys.get(index)?, flags.get(index)?))
        }
    };

    loop {
        let Some(p0) = get_p(i) else {
            break;
        };
        i += 1;
        let Some(p1) = get_p(i) else {
            break;
        };

        if p1.2.on_curve {
            bitmap_maker = draw_straight_line(
                glyph,
                padding,
                variations_of_big,
                height,
                bitmap_maker,
                (p0.0, p0.1),
                (p1.0, p1.1),
            );
        } else {
            i += 1;
            let Some(p2) = get_p(i) else {
                break;
            };

            if !p2.2.on_curve {
                i += 1;
                println!("SHOULD HAVE DRAWN A CURVE COMING UP WITH A MIDDLE POINT");
                // come up with the new middle point and draw curve
                // We nee a p1.5 that is the middle point between p1 and p2
                // Then we draw a curve between p0 and p1.5
                // ???? how do we make it so p1.5 stays in memory as the new p0 in the new loop cycle???
            } else {
                bitmap_maker = draw_curve(
                    glyph,
                    padding,
                    variations_of_big,
                    height,
                    bitmap_maker,
                    (p0.0, p0.1),
                    (p1.0, p1.1),
                    (p2.0, p2.1),
                );
            }
        }
    }

    bitmap_maker
}

fn draw_straight_line(
    glyph: &GlyfData,
    padding: usize,
    variations_of_big: &Vec<(usize, usize)>,
    height: usize,
    mut bitmap_maker: bitmap::BitmapMaker,
    p0: (i16, i16),
    p1: (i16, i16),
) -> bitmap::BitmapMaker {
    let total_distance = distance((p0.0, p0.1), (p1.0, p1.1));

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

        for variations in variations_of_big {
            bitmap_maker = bitmap_maker.with(
                Point {
                    x: (point_on_line.x as i16 - glyph.x_min) as usize + padding / 2 + variations.0,
                    y: height
                        - ((point_on_line.y as i16 - glyph.y_min) as usize
                            + padding / 2
                            + variations.1),
                },
                0x0000FF,
            );
        }
    }

    bitmap_maker
}

fn draw_curve(
    glyph: &GlyfData,
    padding: usize,
    variations_of_big: &Vec<(usize, usize)>,
    height: usize,
    mut bitmap_maker: bitmap::BitmapMaker,
    p0: (i16, i16),
    p1: (i16, i16),
    p2: (i16, i16),
) -> bitmap::BitmapMaker {
    let p0_to_p1_distance = distance((p0.0, p0.1), (p1.0, p1.1));
    let p1_to_p2_distance = distance((p1.0, p1.1), (p2.0, p2.1));
    let total_distance = p0_to_p1_distance + p1_to_p2_distance;

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

    bitmap_maker
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
