#[derive(Clone, Copy)]
pub enum PixelInfo {
    Empty,
    Zero, // SD
    One,  // DS
    InvisibleVertex,
    VisibleVertexZero,
    VisibleVertexOne,
}

// (0,0) is bottom left
pub struct PixelMap {
    pub width: usize,
    pub height: usize,
    map: Vec<PixelInfo>,
}

impl PixelMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            map: vec![PixelInfo::Empty; width * height],
        }
    }

    // (0,0) is bottom left
    pub fn set(&mut self, info: PixelInfo, x: usize, y: usize) {
        match (info, self.map[x + y * self.width]) {
            (PixelInfo::One, PixelInfo::InvisibleVertex)
            | (PixelInfo::Zero, PixelInfo::InvisibleVertex) => {}
            (PixelInfo::One, PixelInfo::Zero) | (PixelInfo::Zero, PixelInfo::One) => {
                self.map[x + y * self.width] = PixelInfo::InvisibleVertex;
            }
            _ => self.map[x + y * self.width] = info,
        }
    }

    pub fn get_unchecked(&self, x: usize, y: usize) -> PixelInfo {
        self.map[x + y * self.width]
    }
}
