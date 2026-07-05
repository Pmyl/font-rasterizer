#[derive(Clone, Copy)]
pub enum PixelInfo {
    Empty,
    Zero, // SD
    One,  // DS
}

pub struct PixelMap {
    width: usize,
    map: Vec<PixelInfo>,
}

impl PixelMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            map: vec![PixelInfo::Empty; width * height],
        }
    }

    pub fn set(&mut self, info: PixelInfo, x: usize, y: usize) {
        self.map[x + y * self.width] = info;
    }

    pub fn get_unchecked(&self, x: usize, y: usize) -> PixelInfo {
        self.map[x + y * self.width]
    }
}
