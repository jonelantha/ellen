#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::video::{MAX_LINES, VideoRegisters};

#[cfg(test)]
mod tests;

pub struct CRTC {
    registers: Rc<RefCell<VideoRegisters>>,
    addr: u16,
    char_line: u16,
    in_char_line_up: u16,
    vsync_state: u8,
    interlace_frame: bool,
    scanline: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotParams {
    pub in_scan: bool,
    pub scanline: u16,
    pub address: u16,
    pub raster_address_even: u8,
    pub raster_address_odd: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdvanceScanlineResult {
    pub field_complete: bool,
    pub next_scanline_trigger: u16,
    pub snapshot_params: SnapshotParams,
    pub vsync: bool,
}

impl CRTC {
    pub fn new(registers: Rc<RefCell<VideoRegisters>>) -> Self {
        CRTC {
            registers,
            addr: 0,
            char_line: 0,
            in_char_line_up: 0,
            vsync_state: 0,
            interlace_frame: false,
            scanline: 0,
        }
    }

    pub fn init(&mut self) {
        let registers = *self.registers.borrow();

        self.char_line = 0;
        self.in_char_line_up = 0;
        self.vsync_state = 0;
        self.interlace_frame = false;
        self.scanline = 0;
        self.addr = calc_screen_address(
            registers.crtc_r12_start_address_h,
            registers.crtc_r13_start_address_l,
        );
    }

    pub fn advance_scanline(&mut self) -> AdvanceScanlineResult {
        let registers = *self.registers.borrow();
        let sync_and_video = registers.r8_is_interlace_sync_and_video();

        let raster_base = self.in_char_line_up * if sync_and_video { 2 } else { 1 };
        let raster_odd_offset = if sync_and_video { 1 } else { 0 };

        let snapshot_params = SnapshotParams {
            in_scan: self.char_line < registers.crtc_r6_vertical_displayed as u16,
            scanline: self.scanline,
            address: self.addr,
            raster_address_even: raster_base as u8,
            raster_address_odd: (raster_base + raster_odd_offset) as u8,
        };

        let mut field_complete = false;

        let char_scanlines = calc_char_scanlines(
            registers.crtc_r9_maximum_raster_address,
            registers.crtc_r8_interlace_and_skew,
        );

        if self.vsync_state == 0
            && self.char_line == registers.crtc_r7_vertical_sync_position as u16
            && self.in_char_line_up == 0
        {
            self.vsync_state = registers.crtc_r3_sync_width >> 4;
            field_complete = true;
        } else if self.vsync_state > 0 {
            self.vsync_state -= 1;
        }

        let vsync = self.vsync_state > 0;

        self.in_char_line_up += 1;
        self.scanline += 1;

        if self.char_line <= registers.crtc_r4_vertical_total as u16
            && self.in_char_line_up >= char_scanlines
        {
            self.char_line += 1;
            self.in_char_line_up = 0;
            self.addr = self
                .addr
                .wrapping_add(registers.crtc_r1_horizontal_displayed as u16);
        }

        let mut next_scanline_trigger = (registers.crtc_r0_horizontal_total as u16 + 1)
            * if calc_is_high_freq(registers.ula_control) {
                1
            } else {
                2
            };

        if self.char_line > registers.crtc_r4_vertical_total as u16
            && self.in_char_line_up >= registers.crtc_r5_vertical_total_adjust as u16
        {
            self.start_of_frame_data(
                registers.crtc_r12_start_address_h,
                registers.crtc_r13_start_address_l,
            );

            if self.interlace_frame && calc_is_interlace(registers.crtc_r8_interlace_and_skew) {
                next_scanline_trigger = next_scanline_trigger.saturating_mul(2);
            }
        }

        if self.scanline == MAX_LINES as u16 {
            field_complete = true;
        }

        if field_complete {
            self.scanline = 0;
        }

        AdvanceScanlineResult {
            field_complete,
            next_scanline_trigger,
            snapshot_params,
            vsync,
        }
    }

    fn start_of_frame_data(&mut self, start_address_h: u8, start_address_l: u8) {
        self.interlace_frame = !self.interlace_frame;
        self.char_line = 0;
        self.in_char_line_up = 0;
        self.addr = calc_screen_address(start_address_h, start_address_l);
    }
}

fn calc_screen_address(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | low as u16
}

fn calc_is_interlace(r8_interlace_and_skew: u8) -> bool {
    r8_interlace_and_skew & 0x01 == 0x01
}

fn calc_is_high_freq(ula_control: u8) -> bool {
    ula_control & 0x10 != 0
}

fn calc_char_scanlines(r9_scan_lines_per_char: u8, r8_interlace_and_skew: u8) -> u16 {
    if r8_interlace_and_skew & 0x03 == 0x03 {
        ((r9_scan_lines_per_char >> 1) + 1) as u16
    } else {
        (r9_scan_lines_per_char + 1) as u16
    }
}
