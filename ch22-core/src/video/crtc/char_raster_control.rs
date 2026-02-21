use crate::video::VideoRegisters;

pub enum CharRasterPosition {
    StartOfField,
    StartOfCharRow,
    VsyncStart,
}

#[derive(Default)]
pub struct CharRasterControl {
    char_row: u8,
    char_raster_in_row: u8,
}

impl CharRasterControl {
    pub fn reset(&mut self) {
        self.char_row = 0;
        self.char_raster_in_row = 0;
    }

    pub fn advance_scanline(&mut self, registers: &VideoRegisters) {
        debug_assert!(registers.crtc_r4_vertical_total <= 0x7F); // r4 should only be 7 bit
        debug_assert!(registers.crtc_r5_vertical_total_adjust <= 0x1F); // r5 should only be 5 bit

        self.char_raster_in_row += 1;

        if self.char_row <= registers.crtc_r4_vertical_total
            && self.char_raster_in_row >= registers.r8_r9_rasters_per_char()
        {
            self.char_row += 1;
            self.char_raster_in_row = 0;
        }

        if self.char_row > registers.crtc_r4_vertical_total
            && self.char_raster_in_row >= registers.crtc_r5_vertical_total_adjust
        {
            self.reset();
        }
    }

    pub fn get_position(&self, registers: &VideoRegisters) -> Option<CharRasterPosition> {
        if self.char_row == 0 && self.char_raster_in_row == 0 {
            Some(CharRasterPosition::StartOfField)
        } else if self.char_raster_in_row == 0 {
            Some(CharRasterPosition::StartOfCharRow)
        } else if self.char_row == registers.crtc_r7_vertical_sync_position
            // after first raster of vsync pos
            && self.char_raster_in_row == 1
        {
            Some(CharRasterPosition::VsyncStart)
        } else {
            None
        }
    }

    pub fn is_at_start(&self) -> bool {
        self.char_row == 0 && self.char_raster_in_row == 0
    }

    pub fn is_in_scan(&self, registers: &VideoRegisters) -> bool {
        self.char_row < registers.crtc_r6_vertical_displayed
    }

    pub fn get_raster_address_even(&self, registers: &VideoRegisters) -> u8 {
        if registers.r8_is_interlace_sync_and_video() {
            self.char_raster_in_row << 1
        } else {
            self.char_raster_in_row
        }
    }

    pub fn get_raster_address_odd(&self, registers: &VideoRegisters) -> u8 {
        if registers.r8_is_interlace_sync_and_video() {
            self.get_raster_address_even(registers) + 1
        } else {
            self.get_raster_address_even(registers)
        }
    }
}
