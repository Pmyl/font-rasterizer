// TODO: Adjustment of "+1 current byte if odd" is a workaround, should read the loca table for the correct offsets
// TODO: use the parser-debug to help in understanding why the fifth glyf fails (that is actually the seventh but we skip the empty ones because we don't read the loca table)
//      The current byte for the glyf seems correct but the flag is invalid, use the debug parser in js to understand why it doesn't fail there
// https://fontdrop.info/#/?darkmode=true

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// xxd -g1 -s 0x7bc -l 268 ./Verdana.ttf | less

/*
000007a8: 00 00 00 02 00 01 00 00 00 00 00 14 00 03 00 01  ................
000007b8: 00 00 01 1a 00 00 01 06 00 00 01 00 00 00 00 00  ................
000007c8: 00 00 01 02 00 00 00 02 00 00 00 00 00 00 00 00  ................
000007d8: 00 00 00 00 00 00 00 01 00 00 03 04 05 06 07 08  ................
000007e8: 09 0a 0b 0c 0d 0e 0f 10 11 12 13 14 15 16 17 18  ................
000007f8: 19 1a 1b 1c 1d 1e 1f 20 21 22 23 24 25 26 27 28  ....... !"#$%&'(
00000808: 29 2a 2b 2c 2d 2e 2f 30 31 32 33 34 35 36 37 38  )*+,-./012345678
00000818: 39 3a 3b 3c 3d 3e 3f 40 41 42 43 44 45 46 47 48  9:;<=>?@ABCDEFGH
00000828: 49 4a 4b 4c 4d 4e 4f 50 51 52 53 54 55 56 57 58  IJKLMNOPQRSTUVWX
00000838: 59 5a 5b 5c 5d 5e 5f 60 61 00 62 63 64 65 66 67  YZ[\]^_`a.bcdefg
00000848: 68 69 6a 6b 6c 6d 6e 6f 70 71 72 73 74 75 76 77  hijklmnopqrstuvw
00000858: 78 79 7a 7b 7c 7d 7e 7f 80 81 82 83 84 85 86 87  xyz{|}~.........
00000868: 88 89 8a 8b 8c 8d 8e 8f 90 91 92 93 94 95 96 97  ................
00000878: 98 99 9a 9b 9c 9d 9e 9f a0 a1 a2 a3 a4 a5 a6 a7  ................
00000888: a8 a9 aa ab 03 ac ad ae af b0 b1 b2 b3 b4 b5 b6  ................
00000898: b7 b8 b9 ba bb bc bd be bf c0 c1 c2 c3 c4 c5 c6  ................
000008a8: c7 c8 c9 ca cb cc cd ce cf d0 00 d1 d2 d3 d4 d5  ................
000008b8: d6 d7 d8 d9 da db dc dd de df 00 04 05 72 00 00  .............r..
000008c8: 00 84 00 80 00 06 00 04 00 7e 01 7f 01 92        .........~....
 */

use std::{
    env,
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::unix::fs::FileExt,
};

use font_rasterizer::{
    OffsetSubtable, TableDirectory, TableDirectoryEntry,
    cmap::{self, Cmap, CmapEncodingSubtable, CmapSubtable, Format0},
    glyf::{Glyf, GlyfData, GlyfDefinition, GlyfFlag, SimpleGlyfDefinition},
    head::Head,
    loca::Loca,
    maxp::{self, Maxp},
};

type Result<T> = core::result::Result<T, Box<dyn Error>>;

// 1960 offset 7a8
// 1676 length 68c
//
fn main() -> Result<()> {
    let filename = env::args()
        .skip(1)
        .next()
        .unwrap_or("Arial.ttf".to_string());
    // .expect("Provide one parameter: font filename");
    dbg!(&filename);

    let mut file = OpenOptions::new().read(true).open(filename)?;

    let offset_subtable = OffsetSubtable {
        scaler_type: read_u32(&mut file).map_err(|e| format!("Reading scaler type {}", e))?,
        number_of_tables: read_u16(&mut file)
            .map_err(|e| format!("Reading number_of_tables {}", e))?,
        search_range: read_u16(&mut file).map_err(|e| format!("Reading search_range {}", e))?,
        entry_selector: read_u16(&mut file).map_err(|e| format!("Reading entry_selector {}", e))?,
        range_shift: read_u16(&mut file).map_err(|e| format!("Reading range_shift {}", e))?,
    };

    let mut table_directory = TableDirectory::new(offset_subtable.number_of_tables as usize);
    for _ in 0..offset_subtable.number_of_tables {
        let entry = TableDirectoryEntry {
            tag: read_bytes::<4>(&mut file).map_err(|e| format!("Reading tag {}", e))?,
            check_sum: read_u32(&mut file).map_err(|e| format!("Reading check_sum {}", e))?,
            offset: read_u32(&mut file).map_err(|e| format!("Reading offset {}", e))?,
            length: read_u32(&mut file).map_err(|e| format!("Reading length {}", e))?,
        };

        table_directory.add_entry(entry);
    }

    dbg!(&offset_subtable);
    dbg!(&table_directory);

    let cmap_entry = table_directory.get(b"cmap")?;
    let cmap = read_cmap(&mut file, cmap_entry.offset, cmap_entry.length)?;
    dbg!(cmap);
    let maxp_entry = table_directory.get(b"maxp")?;
    let maxp = read_maxp(&mut file, maxp_entry.offset, maxp_entry.length)?;
    dbg!(&maxp);
    let head_entry = table_directory.get(b"head")?;
    let head = read_head(&mut file, head_entry)?;
    dbg!(&head);
    let loca_entry = table_directory.get(b"loca")?;
    let loca = read_loca(&mut file, loca_entry, &head, &maxp)?;
    dbg!(&loca);
    let glyf_entry = table_directory.get(b"glyf")?;
    let glyf = read_glyf(&mut file, glyf_entry, &loca)?;
    dbg!(glyf);

    Ok(())
}

// https://learn.microsoft.com/en-us/typography/opentype/spec/glyf
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6glyf.html
fn read_glyf(file: &mut File, entry: &TableDirectoryEntry, loca: &Loca) -> Result<Glyf> {
    file.seek(SeekFrom::Start(entry.offset as u64))
        .map_err(|e| format!("Seeking for glyf {}", e))?;

    let mut glyphs = vec![];
    let mut current_length_in_bytes;
    let mut counter = 0;

    for [start, end] in loca.offsets.array_windows::<2>() {
        let target_offset = entry.offset + start;
        counter += 1;
        dbg!(counter);

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

        let current_byte = file
            .seek(SeekFrom::Start(target_offset as u64))
            .map_err(|e| format!("Seeking for glyf {}", e))?;

        current_length_in_bytes = current_byte as u32 - entry.offset;

        dbg!(current_length_in_bytes);
        println!("Current byte: {}", current_byte);

        let number_of_contours =
            read_i16(file).map_err(|e| format!("Reading number_of_contours {}", e))?;
        current_length_in_bytes += 2;
        let x_min = read_i16(file).map_err(|e| format!("Reading x_min {}", e))?;
        let y_min = read_i16(file).map_err(|e| format!("Reading y_min {}", e))?;
        let x_max = read_i16(file).map_err(|e| format!("Reading x_max {}", e))?;
        let y_max = read_i16(file).map_err(|e| format!("Reading y_max {}", e))?;
        current_length_in_bytes += 8;

        dbg!(number_of_contours);

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
                current_length_in_bytes += 2;
            }

            let instruction_length =
                read_u16(file).map_err(|e| format!("Reading instruction length {}", e))?;
            current_length_in_bytes += 2;

            let mut instructions = Vec::with_capacity(instruction_length as usize);
            for _ in 0..instruction_length {
                instructions
                    .push(read_u8(file).map_err(|e| format!("Reading instructions {}", e))?);
                current_length_in_bytes += 1;
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
                current_length_in_bytes += 1;

                assert!(
                    flag & 0b11000000 == 0b00000000,
                    "flags {:#?} flag {} current length in bytes {}, {}",
                    flags,
                    flag,
                    current_length_in_bytes,
                    instruction_length
                );

                let repeat = flag & 0b00001000 == 0b00001000;
                let number_of_times = if repeat {
                    current_length_in_bytes += 1;
                    read_u8(file).map_err(|e| format!("Reading repeated flag {}", e))?
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
                        current_length_in_bytes += 1;
                    }
                    (true, true) => {
                        x_coordinates.push(
                            read_u8(file).map_err(|e| format!("Reading x coordinate u8 {}", e))?
                                as i16
                                + *x_coordinates.last().unwrap_or(&0),
                        );
                        current_length_in_bytes += 1;
                    }
                    (false, false) => {
                        x_coordinates.push(
                            read_i16(file)
                                .map_err(|e| format!("Reading x coordinate i16 {}", e))?
                                + *x_coordinates.last().unwrap_or(&0),
                        );
                        current_length_in_bytes += 2;
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
                        current_length_in_bytes += 1;
                    }
                    (true, true) => {
                        y_coordinates.push(
                            read_u8(file).map_err(|e| format!("Reading y coordinate u8 {}", e))?
                                as i16
                                + *y_coordinates.last().unwrap_or(&0),
                        );
                        current_length_in_bytes += 1;
                    }
                    (false, false) => {
                        y_coordinates.push(
                            read_i16(file)
                                .map_err(|e| format!("Reading y coordinate i16 {}", e))?
                                + *y_coordinates.last().unwrap_or(&0),
                        );
                        current_length_in_bytes += 2;
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
        // dbg!(&glyph);
        glyphs.push(glyph);

        if current_length_in_bytes >= entry.length {
            break;
        }
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

fn read_u16_at(file: &mut File, at: u32) -> Result<u16> {
    let mut bytes = [0u8; 2];
    file.read_exact_at(&mut bytes, at as u64)?;
    Ok(u16::from_be_bytes(bytes))
}

fn read_u32_at(file: &mut File, at: u32) -> Result<u32> {
    let mut bytes = [0u8; 4];
    file.read_exact_at(&mut bytes, at as u64)?;
    Ok(u32::from_be_bytes(bytes))
}

fn read_bytes_at<const N: usize>(file: &mut File, at: u32) -> Result<[u8; N]> {
    let mut bytes = [0u8; N];
    file.read_exact_at(&mut bytes, at as u64)?;
    Ok(bytes)
}
