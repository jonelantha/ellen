#![allow(dead_code)]

use crate::video::{MAX_LINES, VideoRegisters};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct CRTC {
    address: u16,
    char_row_counter: u8,
    char_raster_counter: u8,
    vsync_line_counter: u8,
    odd_field: bool,
    beam_scanline: u16,
}

pub struct SnapshotParams {
    pub in_scan: bool,
    pub beam_scanline: u16,
    pub address: u16,
    pub raster_address_even: u8,
    pub raster_address_odd: u8,
}

pub struct AdvanceScanlineResult {
    pub field_complete: bool,
    pub next_scanline_trigger: u16,
    pub snapshot_params: SnapshotParams,
    pub vsync: bool,
}

impl CRTC {
    pub fn init(&mut self, registers: &VideoRegisters) {
        self.char_row_counter = 0;
        self.char_raster_counter = 0;
        self.vsync_line_counter = 0;
        self.odd_field = false;
        self.beam_scanline = 0;
        self.address = registers.r12_r13_screen_address();
    }

    pub fn advance_scanline(&mut self, registers: &VideoRegisters) -> AdvanceScanlineResult {
        let snapshot_params = self.get_snapshot_params(registers);

        debug_assert!(registers.crtc_r4_vertical_total <= 0x7F); // r4 should only be 7 bit
        debug_assert!(registers.crtc_r5_vertical_total_adjust <= 0x1F); // r5 should only be 5 bit

        let rasters_per_char = registers.r8_r9_rasters_per_char();

        self.beam_scanline += 1;

        if self.vsync_line_counter == 0
            && self.char_row_counter == registers.crtc_r7_vertical_sync_position
            && self.char_raster_counter == 0
        {
            self.vsync_line_counter = registers.r3_v_sync_width();
            self.beam_scanline = 0;
        } else if self.vsync_line_counter > 0 {
            self.vsync_line_counter -= 1;
        }

        if self.beam_scanline == MAX_LINES as u16 {
            self.beam_scanline = 0;
        }

        self.char_raster_counter += 1;

        if self.char_row_counter <= registers.crtc_r4_vertical_total
            && self.char_raster_counter >= rasters_per_char
        {
            self.char_row_counter += 1;
            self.char_raster_counter = 0;
            self.address = (self.address + registers.crtc_r1_horizontal_displayed as u16) & 0x3FFF;
        }

        if self.char_row_counter > registers.crtc_r4_vertical_total
            && self.char_raster_counter >= registers.crtc_r5_vertical_total_adjust
        {
            // start new field
            self.odd_field = !self.odd_field;
            self.char_row_counter = 0;
            self.char_raster_counter = 0;
            self.address = registers.r12_r13_screen_address();
        }

        AdvanceScanlineResult {
            field_complete: self.beam_scanline == 0,
            next_scanline_trigger: self.get_next_scanline_trigger(registers),
            snapshot_params,
            vsync: self.vsync_line_counter > 0,
        }
    }

    fn get_snapshot_params(&self, registers: &VideoRegisters) -> SnapshotParams {
        let sync_and_video = registers.r8_is_interlace_sync_and_video();

        let (raster_address_even, raster_address_odd) = if sync_and_video {
            (
                self.char_raster_counter << 1,
                (self.char_raster_counter << 1) + 1,
            )
        } else {
            (self.char_raster_counter, self.char_raster_counter)
        };

        SnapshotParams {
            in_scan: self.char_row_counter < registers.crtc_r6_vertical_displayed,
            beam_scanline: self.beam_scanline,
            address: self.address,
            raster_address_even,
            raster_address_odd,
        }
    }

    fn get_next_scanline_trigger(&self, registers: &VideoRegisters) -> u16 {
        let mut next_scanline_trigger = registers.crtc_r0_horizontal_total as u16 + 1;

        if !registers.ula_is_high_frequency() {
            next_scanline_trigger *= 2;
        }

        if self.char_row_counter == 0
            && self.char_raster_counter == 0
            && self.odd_field
            && registers.r8_is_interlace()
        {
            next_scanline_trigger *= 2;
        }

        next_scanline_trigger
    }
}
