use crate::font::mac_os_roman::from_byte_to_cmap_index;

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
    pub seg_count: u16,
    pub end_codes: Vec<u16>,
    pub start_codes: Vec<u16>,
    pub id_deltas: Vec<u16>,
    pub id_range_offsets: Vec<u16>,
    pub glyph_index_array: Vec<u16>, // An array that maps character codes to glyph index values
}

impl Cmap {
    pub fn get_glyph_index(&self, c: char) -> Option<usize> {
        self.subtables
            .iter()
            .find_map(|subtable| subtable.get_glyph_index(c))
    }
}

impl CmapSubtable {
    fn get_glyph_index(&self, c: char) -> Option<usize> {
        match self {
            CmapSubtable::Format0(_) => from_byte_to_cmap_index(c),
            CmapSubtable::Format4(format4) => {
                let c = Into::<u32>::into(c) as usize;
                let i = format4.end_codes.iter().position(|e| *e >= c as u16)?;
                if format4.start_codes[i] > c as u16 {
                    return None;
                }

                // Fallback, we don't need to use id range offsets, id delta is enough
                if format4.id_range_offsets[i] == 0 {
                    return Some((format4.id_deltas[i] as usize + c).rem_euclid(65536));
                }

                // Use id range offset
                // *(&idRangeOffset[i] + idRangeOffset[i] / 2 + (c - startCode[i]))

                let segments = format4.seg_count as usize;
                let offset = (format4.id_range_offsets[i] / 2 + (c as u16 - format4.start_codes[i]))
                    as usize;

                let glyph_index_array_index = offset - (segments - i);
                Some(format4.glyph_index_array[glyph_index_array_index as usize] as usize)
            }
            CmapSubtable::Unhandled { .. } => None,
        }
    }
}
