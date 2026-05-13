use crate::{cmap::Cmap, glyf::Glyf};

pub mod cmap;
pub mod glyf;

#[derive(Debug)]
pub struct TrueTypeFont {
    pub offset_subtable: OffsetSubtable,
    pub table_directory: Vec<TableDirectoryEntry>,
    pub cmap: Cmap,
    pub glyf: Glyf,
}

#[derive(Debug)]
pub struct OffsetSubtable {
    pub scaler_type: u32,      // expected 0x00010000 for TrueType fonts
    pub number_of_tables: u16, // not including "offset subtable" (the first table)

    // following 3 props are only used to facilitate quick binary search, ignore
    pub search_range: u16,   // (maximum power of 2 <= numTables)*16
    pub entry_selector: u16, // log2(maximum power of 2 <= numTables)
    pub range_shift: u16,    // numTables*16-searchRange
}

#[derive(Debug)]
pub struct TableDirectoryEntry {
    pub tag: [u8; 4],   // in ascending order in the vec
    pub check_sum: u32, // we don't care about integrity
    pub offset: u32,    // offset from beginning of sfnt
    pub length: u32,    // length of this table in byte (actual length not padded length)
}
