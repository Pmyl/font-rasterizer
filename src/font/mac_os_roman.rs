// Simplified version of mapping, takes in account only from 0x00 to 0x7F as it's identical to standard ASCII's first 127 characters
// Takes in account only a single byte instead of full unicode, for full unicode we will need to take a `char` in input instead
pub fn from_byte_to_cmap_index(c: char) -> usize {
    if c.is_ascii() {
        c as usize
    } else {
        0 // Defaults to 0 that is always the unknown/square character
    }
}
