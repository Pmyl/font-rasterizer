#[derive(Debug)]
pub struct Cmap {
    pub version: u16, // ignore
    pub number_of_subtables: u16,
    pub encoding_subtables: Vec<CmapEncodingSubtable>,
    pub subtables: Vec<CmapSubtable>,
}

#[derive(Debug)]
pub struct CmapEncodingSubtable {
    pub platform_id: u16, // 0: Unicode, 1: Macintosh or 3: Microsoft
    pub platform_specific_id: u16,
    pub offset: u32, // offset from the start of Cmap
}

#[derive(Debug)]
pub enum CmapSubtable {
    Format0(Format0),
    Format4(Format4),
    Unhandled { format: u16 },
}

#[derive(Debug)]
pub struct Format0 {
    pub format: u16, // set to 0
    pub length_in_bytes: u16,
    pub language: u16,                // language code
    pub glyph_index_array: [u8; 256], // An array that maps character codes to glyph index values
}

#[derive(Debug)]
pub struct Format4 {
    pub format: u16, // set to 4
    pub length_in_bytes: u16,
    pub language: u16, // language code
    pub seg_count_x2: u16,
    pub end_codes: Vec<u16>,
    pub start_codes: Vec<u16>,
    pub id_deltas: Vec<u16>,
    pub id_range_offsets: Vec<u16>,
    pub glyph_index_array: Vec<u16>, // An array that maps character codes to glyph index values
}
