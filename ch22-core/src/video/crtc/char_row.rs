use crate::video::VideoRegisters;

pub enum CharRowPosition {
    StartOfField,
    StartOfCharRow,
    VsyncStart,
}

#[derive(Default)]
pub struct CharRowControl {
    char_row: u8,
    char_raster: u8,
}

impl CharRowControl {
    pub fn reset(&mut self) {
        self.char_row = 0;
        self.char_raster = 0;
    }

    pub fn advance_scanline(&mut self, registers: &VideoRegisters) {
        debug_assert!(registers.crtc_r4_vertical_total <= 0x7F); // r4 should only be 7 bit
        debug_assert!(registers.crtc_r5_vertical_total_adjust <= 0x1F); // r5 should only be 5 bit

        self.char_raster += 1;

        if self.char_row <= registers.crtc_r4_vertical_total
            && self.char_raster >= registers.r8_r9_rasters_per_char()
        {
            self.char_row += 1;
            self.char_raster = 0;
        }

        if self.char_row > registers.crtc_r4_vertical_total
            && self.char_raster >= registers.crtc_r5_vertical_total_adjust
        {
            self.reset();
        }
    }

    pub fn get_position(&self, registers: &VideoRegisters) -> Option<CharRowPosition> {
        if self.char_row == 0 && self.char_raster == 0 {
            Some(CharRowPosition::StartOfField)
        } else if self.char_raster == 0 {
            Some(CharRowPosition::StartOfCharRow)
        } else if self.char_row == registers.crtc_r7_vertical_sync_position
            // after first raster of vsync pos
            && self.char_raster == 1
        {
            Some(CharRowPosition::VsyncStart)
        } else {
            None
        }
    }

    pub fn is_at_start(&self) -> bool {
        self.char_row == 0 && self.char_raster == 0
    }

    pub fn is_in_scan(&self, registers: &VideoRegisters) -> bool {
        self.char_row < registers.crtc_r6_vertical_displayed
    }

    pub fn get_raster_address_even(&self, registers: &VideoRegisters) -> u8 {
        if registers.r8_is_interlace_sync_and_video() {
            self.char_raster << 1
        } else {
            self.char_raster
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
