pub struct TrueTypeFont {
    offset_subtable: OffsetSubtable,
    table_directory: Vec<TableDirectoryEntry>,
}

pub struct OffsetSubtable {
    scaler_type: u32,      // expected 0x00010000 for TrueType fonts
    number_of_tables: u16, // not including "offset subtable" (the first table)

    // following 3 props are only used to facilitate quick binary search, ignore
    search_range: u16,   // (maximum power of 2 <= numTables)*16
    entry_selector: u16, // log2(maximum power of 2 <= numTables)
    range_shift: u16,    // numTables*16-searchRange
}

pub struct TableDirectoryEntry {
    tag: u32,       // in ascending order in the vec
    check_sum: u32, // we don't care about integrity
    offset: u32,    // offset from beginning of sfnt
    length: u32,    // length of this table in byte (actual length not padded length)
}

struct Cmap {
    version: u16, // ignore
    number_of_subtables: u16,
    encoding_subtables: Vec<CmapEncodingSubtable>,
    subtables: Vec<CmapSubtable>,
}

struct CmapEncodingSubtable {
    platform_id: u16, // 0: Unicode, 1: Macintosh or 3: Microsoft
    platform_specific_id: u16,
    offset: u32, // offset from the start of Cmap
}

struct CmapSubtable {
    format: u16, // set to 0
    length_in_bytes: u16,
    language: u16,                // language code
    glyph_index_array: [u8; 256], // An array that maps character codes to glyph index values
}

// Il Cmap di Verdana inizia a 000007a8
// Il primo subtable punta ad un offset di 0x14 che aggiunto a 0x7a8 fa 0x7bc
// A 0x7bc abbiamo questi bytes iniziali: 00 00 01 06 00 00
// I primi due bytes (00 00) sono il formato, che e' format 0.
// Dalla documentazione PERO' dice che format 0 e' raro, siamo sicuri di aver
//  trovato i bytes giusti?
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

// 1960 offset 7a8
// 1676 length 68c
//
fn main() {
    println!("Hello, world!");
}
