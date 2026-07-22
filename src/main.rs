// https://fontdrop.info/#/?darkmode=true

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// xxd -g1 -s 0x7bc -l 268 ./Verdana.ttf | less

use std::{
    env,
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use font_rasterizer::font::{
    OffsetSubtable, TableDirectory, TableDirectoryEntry, TrueTypeFont,
    cmap::{Cmap, CmapEncodingSubtable, CmapSubtable, Format0, Format4},
    glyf::{Glyf, GlyfData, GlyfDefinition, GlyfFlag, SimpleGlyfDefinition},
    head::Head,
    loca::Loca,
    mac_os_roman::from_byte_to_cmap_index,
    maxp::Maxp,
};

use font_rasterizer::rasterizer::rasterize_glyph_to_bitmap;

type Result<T> = core::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut args = env::args();
    args.next(); // skip binary name

    let filename = args
        .next()
        .expect("Provide the first parameter: font filename");

    let character_to_show = args
        .next()
        .unwrap_or("g".to_string())
        .chars()
        .next()
        .unwrap();

    let target_file_name = args.next().unwrap_or("images/image.bmp".to_string());

    let target_file_path = Path::new(&target_file_name);

    let mut file = OpenOptions::new().read(true).open(filename)?;

    let font = file_to_true_type_font(&mut file)?;

    // dbg!(&font.cmap);

    let mut printed = false;

    for subtable in &font.cmap.subtables {
        match &subtable {
            CmapSubtable::Format0(format0) => {
                let cmap_index = from_byte_to_cmap_index(character_to_show);
                let index = format0.glyph_index_array[cmap_index];
                let glyph = &font.glyf.glyphs[index as usize];
                // println!("Index {:#?} -> {} -> {}", glyph, cmap_index, index);

                rasterize_glyph_to_bitmap(glyph, &target_file_path);
                printed = true;
                break;
            }
            CmapSubtable::Format4(format4) => todo!(),
            CmapSubtable::Unhandled { .. } => {}
        }
    }

    if !printed {
        eprintln!(
            "Format0 not found, only these formats exist: {:?}",
            font.cmap.subtables
        );
    }

    Ok(())
}

fn file_to_true_type_font(file: &mut File) -> Result<TrueTypeFont> {
    let offset_subtable = OffsetSubtable {
        scaler_type: read_u32(file).map_err(|e| format!("Reading scaler type {}", e))?,
        number_of_tables: read_u16(file).map_err(|e| format!("Reading number_of_tables {}", e))?,
        search_range: read_u16(file).map_err(|e| format!("Reading search_range {}", e))?,
        entry_selector: read_u16(file).map_err(|e| format!("Reading entry_selector {}", e))?,
        range_shift: read_u16(file).map_err(|e| format!("Reading range_shift {}", e))?,
    };

    let mut table_directory = TableDirectory::new(offset_subtable.number_of_tables as usize);
    for _ in 0..offset_subtable.number_of_tables {
        let entry = TableDirectoryEntry {
            tag: read_bytes::<4>(file).map_err(|e| format!("Reading tag {}", e))?,
            check_sum: read_u32(file).map_err(|e| format!("Reading check_sum {}", e))?,
            offset: read_u32(file).map_err(|e| format!("Reading offset {}", e))?,
            length: read_u32(file).map_err(|e| format!("Reading length {}", e))?,
        };

        table_directory.add_entry(entry);
    }

    // dbg!(&offset_subtable);
    // dbg!(&table_directory);

    let cmap_entry = table_directory.get(b"cmap")?;
    let cmap = read_cmap(file, cmap_entry.offset, cmap_entry.length)?;
    let maxp_entry = table_directory.get(b"maxp")?;
    let maxp = read_maxp(file, maxp_entry.offset, maxp_entry.length)?;
    let head_entry = table_directory.get(b"head")?;
    let head = read_head(file, head_entry)?;
    let loca_entry = table_directory.get(b"loca")?;
    let loca = read_loca(file, loca_entry, &head, &maxp)?;
    let glyf_entry = table_directory.get(b"glyf")?;
    let glyf = read_glyf(file, glyf_entry, &loca)?;

    let font = TrueTypeFont {
        offset_subtable,
        table_directory,
        cmap,
        glyf,
    };

    // dbg!(&font.cmap);
    return Ok(font);
}

fn read_glyf(file: &mut File, entry: &TableDirectoryEntry, loca: &Loca) -> Result<Glyf> {
    file.seek(SeekFrom::Start(entry.offset as u64))
        .map_err(|e| format!("Seeking for glyf {}", e))?;

    let mut glyphs = vec![];

    for [start, end] in loca.offsets.array_windows::<2>() {
        if start == end {
            let definition: GlyfDefinition = GlyfDefinition::Simple(SimpleGlyfDefinition {
                end_pts_of_contours: vec![],
                instruction_length: 0,
                instructions: vec![],
                flags: vec![],
                x_coordinates: vec![],
                y_coordinates: vec![],
            });
            let glyph = GlyfData {
                number_of_contours: 0,
                x_min: 0,
                y_min: 0,
                x_max: 0,
                y_max: 0,
                definition,
            };

            glyphs.push(glyph);
            continue;
        }

        file.seek(SeekFrom::Start((entry.offset + start) as u64))
            .map_err(|e| format!("Seeking for glyf {}", e))?;

        let number_of_contours =
            read_i16(file).map_err(|e| format!("Reading number_of_contours {}", e))?;
        let x_min = read_i16(file).map_err(|e| format!("Reading x_min {}", e))?;
        let y_min = read_i16(file).map_err(|e| format!("Reading y_min {}", e))?;
        let x_max = read_i16(file).map_err(|e| format!("Reading x_max {}", e))?;
        let y_max = read_i16(file).map_err(|e| format!("Reading y_max {}", e))?;

        let definition: GlyfDefinition = if number_of_contours < 0 {
            GlyfDefinition::Compound
        } else if number_of_contours == 0 {
            let simple = SimpleGlyfDefinition {
                end_pts_of_contours: vec![],
                instruction_length: 0,
                instructions: vec![],
                flags: vec![],
                x_coordinates: vec![],
                y_coordinates: vec![],
            };

            GlyfDefinition::Simple(simple)
        } else {
            let mut end_pts_of_contours: Vec<u16> = Vec::with_capacity(number_of_contours as usize);
            for _ in 0..number_of_contours {
                end_pts_of_contours
                    .push(read_u16(file).map_err(|e| format!("Reading end pts {}", e))?);
            }

            let instruction_length =
                read_u16(file).map_err(|e| format!("Reading instruction length {}", e))?;

            let mut instructions = Vec::with_capacity(instruction_length as usize);
            for _ in 0..instruction_length {
                instructions
                    .push(read_u8(file).map_err(|e| format!("Reading instructions {}", e))?);
            }

            let number_of_points = *end_pts_of_contours.last().expect("At least one contour") + 1;

            let mut flags: Vec<GlyfFlag> = Vec::with_capacity(number_of_points as usize);
            loop {
                if flags.len() == number_of_points as usize {
                    break;
                }
                assert!(
                    flags.len() < number_of_points as usize,
                    "Flags: {} Points: {}",
                    flags.len(),
                    number_of_points
                );

                /*
                On Curve 	    0 	If set, the point is on the curve;
                x-Short Vector 	1 	If set, the corresponding x-coordinate is 1 byte long;
                y-Short Vector 	2 	If set, the corresponding y-coordinate is 1 byte long;
                Repeat 	        3 	If set, the next byte specifies the number of additional times this set of flags is to be repeated. In this way, the number of flags listed can be smaller than the number of points in a character.
                This x is same 	4 	This flag has one of two meanings, depending on how the x-Short Vector flag is set.
                This y is same 	5 	This flag has one of two meanings, depending on how the y-Short Vector flag is set.
                Reserved        6-7
                */

                let flag = read_u8(file).map_err(|e| format!("Reading flag {}", e))?;

                assert!(flag & 0b11000000 == 0b00000000);

                let repeat = flag & 0b00001000 == 0b00001000;
                let number_of_times = if repeat {
                    read_u8(file).map_err(|e| format!("Reading repeated flag {}", e))? + 1
                } else {
                    1
                };

                for _ in 0..number_of_times {
                    flags.push(GlyfFlag {
                        on_curve: flag & 0b00000001 == 0b00000001,
                        x_short_vector: flag & 0b00000010 == 0b00000010,
                        y_short_vector: flag & 0b00000100 == 0b00000100,
                        this_x_is_same: flag & 0b00010000 == 0b00010000,
                        this_y_is_same: flag & 0b00100000 == 0b00100000,
                        original_flag: flag,
                    });
                }
            }

            let mut x_coordinates = Vec::with_capacity(number_of_points as usize);
            let mut y_coordinates = Vec::with_capacity(number_of_points as usize);
            for flag in &flags {
                match (flag.x_short_vector, flag.this_x_is_same) {
                    (true, false) => {
                        x_coordinates.push(
                            *x_coordinates.last().unwrap_or(&0)
                                - read_u8(file)
                                    .map_err(|e| format!("Reading x coordinate u8 {}", e))?
                                    as i16,
                        );
                    }
                    (true, true) => {
                        x_coordinates.push(
                            read_u8(file).map_err(|e| format!("Reading x coordinate u8 {}", e))?
                                as i16
                                + *x_coordinates.last().unwrap_or(&0),
                        );
                    }
                    (false, false) => {
                        x_coordinates.push(
                            read_i16(file)
                                .map_err(|e| format!("Reading x coordinate i16 {}", e))?
                                + *x_coordinates.last().unwrap_or(&0),
                        );
                    }
                    (false, true) => {
                        // Sometimes this can be on the first? Default to 0 seems to be fine
                        x_coordinates.push(*x_coordinates.last().unwrap_or(&0));
                    }
                }
            }
            for flag in &flags {
                match (flag.y_short_vector, flag.this_y_is_same) {
                    (true, false) => {
                        y_coordinates.push(
                            *y_coordinates.last().unwrap_or(&0)
                                - read_u8(file)
                                    .map_err(|e| format!("Reading y coordinate u8 {}", e))?
                                    as i16,
                        );
                    }
                    (true, true) => {
                        y_coordinates.push(
                            read_u8(file).map_err(|e| format!("Reading y coordinate u8 {}", e))?
                                as i16
                                + *y_coordinates.last().unwrap_or(&0),
                        );
                    }
                    (false, false) => {
                        y_coordinates.push(
                            read_i16(file)
                                .map_err(|e| format!("Reading y coordinate i16 {}", e))?
                                + *y_coordinates.last().unwrap_or(&0),
                        );
                    }
                    (false, true) => {
                        // Sometimes this can be on the first? Default to 0 seems to be fine
                        y_coordinates.push(*y_coordinates.last().unwrap_or(&0));
                    }
                }
            }

            let simple = SimpleGlyfDefinition {
                end_pts_of_contours,
                instruction_length,
                instructions,
                flags,
                x_coordinates,
                y_coordinates,
            };

            GlyfDefinition::Simple(simple)
        };

        let glyph = GlyfData {
            number_of_contours,
            x_min,
            y_min,
            x_max,
            y_max,
            definition,
        };
        glyphs.push(glyph);
    }

    Ok(Glyf { glyphs })
}

fn read_maxp(file: &mut File, offset: u32, _: u32) -> Result<Maxp> {
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(|e| format!("Seeking for cmap {}", e))?;

    let version = read_u32(file).map_err(|e| format!("Reading maxp version {}", e))?;
    let number_of_glyphs = read_u16(file).map_err(|e| format!("Reading number of glyphs {}", e))?;

    Ok(Maxp {
        version,
        number_of_glyphs,
    })
}

fn read_loca(
    file: &mut File,
    entry: &TableDirectoryEntry,
    head: &Head,
    maxp: &Maxp,
) -> Result<Loca> {
    file.seek(SeekFrom::Start(entry.offset as u64))
        .map_err(|e| format!("Seeking for loca {}", e))?;

    let mut offsets = Vec::with_capacity(maxp.number_of_glyphs as usize);
    for _ in 0..maxp.number_of_glyphs {
        if head.index_to_loc_format == 0 {
            offsets.push(read_u16(file)? as u32 * 2);
        } else {
            offsets.push(read_u32(file)?);
        }
    }

    Ok(Loca { offsets })
}

fn read_head(file: &mut File, entry: &TableDirectoryEntry) -> Result<Head> {
    file.seek(SeekFrom::Start(entry.offset as u64))
        .map_err(|e| format!("Seeking for head {}", e))?;

    Ok(Head {
        version: read_u32(file).map_err(|e| format!("Reading version {}", e))?,
        font_revision: read_i32(file).map_err(|e| format!("Reading font_revision {}", e))?,
        checksum_adjustment: read_u32(file)
            .map_err(|e| format!("Reading checksum_adjustment {}", e))?,
        magic_number: read_u32(file).map_err(|e| format!("Reading magic_number {}", e))?,
        flags: read_u16(file).map_err(|e| format!("Reading flags {}", e))?,
        units_per_em: read_u16(file).map_err(|e| format!("Reading units_per_em {}", e))?,
        created: read_u64(file).map_err(|e| format!("Reading created {}", e))?,
        modified: read_u64(file).map_err(|e| format!("Reading modified {}", e))?,
        xmin: read_i16(file).map_err(|e| format!("Reading xmin {}", e))?,
        ymin: read_i16(file).map_err(|e| format!("Reading ymin {}", e))?,
        xmax: read_i16(file).map_err(|e| format!("Reading xmax {}", e))?,
        ymax: read_i16(file).map_err(|e| format!("Reading ymax {}", e))?,
        mac_style: read_u16(file).map_err(|e| format!("Reading mac_style {}", e))?,
        lowest_rec_ppem: read_u16(file).map_err(|e| format!("Reading lowest_rec_ppem {}", e))?,
        font_direction_hint: read_i16(file)
            .map_err(|e| format!("Reading font_direction_hint {}", e))?,
        index_to_loc_format: read_i16(file)
            .map_err(|e| format!("Reading index_to_loc_format {}", e))?,
        glyph_data_format: read_i16(file)
            .map_err(|e| format!("Reading glyph_data_format {}", e))?,
    })
}

fn read_cmap(file: &mut File, offset: u32, _: u32) -> Result<Cmap> {
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(|e| format!("Seeking for cmap {}", e))?;

    let version = read_u16(file).map_err(|e| format!("Reading version {}", e))?;
    let number_of_subtables =
        read_u16(file).map_err(|e| format!("Reading number_of_subtables {}", e))?;

    let mut encoding_subtables = Vec::with_capacity(number_of_subtables as usize);
    for _ in 0..number_of_subtables {
        let subtable = CmapEncodingSubtable {
            platform_id: read_u16(file).map_err(|e| format!("Reading platform_id {}", e))?,
            platform_specific_id: read_u16(file)
                .map_err(|e| format!("Reading platform_specific_id {}", e))?,
            offset: read_u32(file).map_err(|e| format!("Reading offset {}", e))?,
        };

        encoding_subtables.push(subtable);
    }

    let mut subtables = Vec::with_capacity(number_of_subtables as usize);
    for encoding_subtable in &encoding_subtables {
        file.seek(SeekFrom::Start((offset + encoding_subtable.offset) as u64))
            .map_err(|e| format!("Seeking for cmap {}", e))?;

        let format = read_u16(file).map_err(|e| format!("Reading format {}", e))?;
        let subtable = if format == 0 {
            let format0 = Format0 {
                format: 0,
                length_in_bytes: read_u16(file)
                    .map_err(|e| format!("Reading length_in_bytes {}", e))?,
                language: read_u16(file).map_err(|e| format!("Reading language {}", e))?,
                glyph_index_array: read_bytes::<256>(file)
                    .map_err(|e| format!("Reading glyph_index_array {}", e))?,
            };

            assert_eq!(format0.length_in_bytes, 262);
            CmapSubtable::Format0(format0)
        } else if format == 4 {
            let length_in_bytes =
                read_u16(file).map_err(|e| format!("Reading length_in_bytes {}", e))?;
            let language = read_u16(file).map_err(|e| format!("Reading language {}", e))?;
            let seg_count_x2 = read_u16(file).map_err(|e| format!("Reading seg count x2 {}", e))?;
            let _ = read_u16(file).map_err(|e| format!("Reading search range {}", e))?;
            let _ = read_u16(file).map_err(|e| format!("Reading entry selector {}", e))?;
            let _ = read_u16(file).map_err(|e| format!("Reading range shift {}", e))?;
            let end_codes = read_vec_u16(file, seg_count_x2 as usize / 2)
                .map_err(|e| format!("Reading end codes {}", e))?;

            let _ = read_u16(file).map_err(|e| format!("Reading reserved pad {}", e))?;

            let start_codes = read_vec_u16(file, seg_count_x2 as usize / 2)
                .map_err(|e| format!("Reading start codes {}", e))?;

            let id_deltas = read_vec_u16(file, seg_count_x2 as usize / 2)
                .map_err(|e| format!("Reading id deltas {}", e))?;

            let id_range_offsets = read_vec_u16(file, seg_count_x2 as usize / 2)
                .map_err(|e| format!("Reading id range offset {}", e))?;

            let glyph_index_array = todo!();

            CmapSubtable::Format4(Format4 {
                format,
                length_in_bytes,
                language,
                seg_count_x2,
                end_codes,
                start_codes,
                id_deltas,
                id_range_offsets,
                glyph_index_array,
            })
        } else {
            CmapSubtable::Unhandled { format }
        };
        subtables.push(subtable);
    }

    Ok(Cmap {
        version,
        number_of_subtables,
        encoding_subtables,
        subtables,
    })
}

fn read_u8(file: &mut File) -> Result<u8> {
    let mut bytes = [0u8; 1];
    file.read_exact(&mut bytes)?;
    Ok(u8::from_be_bytes(bytes))
}

fn read_u16(file: &mut File) -> Result<u16> {
    let mut bytes = [0u8; 2];
    file.read_exact(&mut bytes)?;
    Ok(u16::from_be_bytes(bytes))
}

fn read_i16(file: &mut File) -> Result<i16> {
    let mut bytes = [0u8; 2];
    file.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

fn read_u32(file: &mut File) -> Result<u32> {
    let mut bytes = [0u8; 4];
    file.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}

fn read_i32(file: &mut File) -> Result<i32> {
    let mut bytes = [0u8; 4];
    file.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}

fn read_u64(file: &mut File) -> Result<u64> {
    let mut bytes = [0u8; 8];
    file.read_exact(&mut bytes)?;
    Ok(u64::from_be_bytes(bytes))
}

fn read_bytes<const N: usize>(file: &mut File) -> Result<[u8; N]> {
    let mut bytes = [0u8; N];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_vec_u16(file: &mut File, length: usize) -> Result<Vec<u16>> {
    let mut vec: Vec<u16> = Vec::with_capacity(length);

    for _ in 0..length {
        let bytes = read_u16(file)?;
        vec.push(bytes);
    }

    Ok(vec)
}
