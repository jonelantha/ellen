use crate::video::MAX_LINES;

#[derive(Default)]
pub struct BeamControl {
    scanline: u16,
}

impl BeamControl {
    pub fn advance_scanline(&mut self) {
        self.scanline += 1;

        if self.scanline == MAX_LINES as u16 {
            self.scanline = 0;
        }
    }

    pub fn reset(&mut self) {
        self.scanline = 0;
    }

    pub fn get_scanline(&self) -> u16 {
        self.scanline
    }
}
