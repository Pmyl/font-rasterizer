#[derive(Debug)]
pub struct Glyf {
    pub glyphs: Vec<GlyfData>,
}

#[derive(Debug)]
pub struct GlyfData {
    // If the number of contours is positive or zero, it is a single glyph;
    // If the number of contours less than zero, the glyph is compound
    pub number_of_contours: i16,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
    pub definition: GlyfDefinition,
}

#[derive(Debug)]
pub enum GlyfDefinition {
    Simple(SimpleGlyfDefinition),
    Compound(CompoundGlyfDefinition),
}

#[derive(Debug)]
pub struct SimpleGlyfDefinition {
    // end_pts_of_contours define the last point index of each contour
    // with this we can infer the number of points for each contour
    pub end_pts_of_contours: Vec<u16>, // size is number_of_contours
    pub instruction_length: u16,
    pub instructions: Vec<u8>, // size is instruction_length
    pub flags: Vec<GlyfFlag>,  // size is <= number of points
    // size is number of points
    pub x_coordinates: Vec<i16>, // can be stored as u8 or i16 based on flags
    // size is number of points
    pub y_coordinates: Vec<i16>, // can be stored as u8 or i16 based on flags
}

#[derive(Debug)]
pub struct GlyfFlag {
    pub on_curve: bool,
    pub x_short_vector: bool,
    pub y_short_vector: bool,
    pub this_x_is_same: bool,
    pub this_y_is_same: bool,
    pub original_flag: u8,
}

#[derive(Debug)]
pub struct CompoundGlyfDefinition {
    pub components: Vec<CompoundGlyf>,
}

#[derive(Debug)]
pub struct CompoundGlyf {
    flags: CompoundGlyFlag,
    glyph_index: u16,
    argument_1: u16,
    argument_2: u16,
}

#[derive(Debug)]
pub struct CompoundGlyFlag {
    pub arg1_and_2_are_words: bool,
    pub args_are_xy_values: bool,
    pub round_xy_to_grid: bool,
    pub we_have_a_scale: bool,
    pub more_components: bool,
    pub we_have_an_x_and_y_scale: bool,
    pub we_have_a_two_by_two: bool,
    pub we_have_instructions: bool,
    pub use_my_metrics: bool,
    pub overlap_compound: bool,
}
