#![allow(dead_code)]

use crate::video::VideoRegisters;

mod address_control;
mod beam_control;
mod char_raster_control;
mod vsync_control;

use self::address_control::AddressControl;
use self::beam_control::BeamControl;
use self::char_raster_control::{CharRasterControl, CharRasterPosition};
use self::vsync_control::VSyncControl;

#[cfg(test)]
mod tests;

pub struct SnapshotParams {
    pub is_displayed: bool,
    pub beam_scanline: u16,
    pub address: u16,
    pub raster_address_even: u8,
    pub raster_address_odd: u8,
}

#[derive(Default)]
pub struct Crtc {
    char_raster_control: CharRasterControl,
    address_control: AddressControl,
    vsync_control: VSyncControl,
    beam_control: BeamControl,
    odd_field: bool,
}

impl Crtc {
    pub fn init(&mut self, registers: &VideoRegisters) {
        self.char_raster_control.reset();
        self.address_control.reset(registers);
        self.beam_control.reset();
        self.vsync_control.reset();
        self.odd_field = false;
    }

    pub fn advance_scanline(&mut self, registers: &VideoRegisters) {
        self.beam_control.advance_scanline();

        self.vsync_control.advance_scanline();

        self.char_raster_control.advance_scanline(registers);

        if let Some(position) = self.char_raster_control.get_position(registers) {
            match position {
                CharRasterPosition::StartOfField => {
                    self.address_control.reset(registers);
                    self.odd_field = !self.odd_field;
                }
                CharRasterPosition::StartOfCharRow => {
                    self.address_control.advance_char_row(registers);
                }
                CharRasterPosition::VsyncStart => {
                    if self.vsync_control.start_vsync_period(registers) {
                        self.beam_control.reset();
                    }
                }
            }
        }
    }

    pub fn is_beam_reset(&self) -> bool {
        self.beam_control.get_scanline() == 0
    }

    pub fn is_in_vsync(&self) -> bool {
        self.vsync_control.is_in_vsync()
    }

    pub fn get_snapshot_params(&self, registers: &VideoRegisters) -> SnapshotParams {
        SnapshotParams {
            is_displayed: self.char_raster_control.is_displayed(registers),
            beam_scanline: self.beam_control.get_scanline(),
            address: self.address_control.get_address(),
            raster_address_even: self.char_raster_control.get_raster_address_even(registers),
            raster_address_odd: self.char_raster_control.get_raster_address_odd(registers),
        }
    }

    pub fn get_next_scanline_cycles(&self, registers: &VideoRegisters) -> u64 {
        let mut next_scanline_trigger = registers.crtc_r0_horizontal_total as u64 + 1;

        if !registers.ula_is_high_frequency() {
            next_scanline_trigger *= 2;
        }

        // in interlace mode, each field is offset by half a raster
        // so a delay for the two halves is inserted every other field
        // (this is an approximation)
        if self.char_raster_control.is_at_start() && self.odd_field && registers.r8_is_interlace() {
            next_scanline_trigger *= 2;
        }

        next_scanline_trigger
    }
}
