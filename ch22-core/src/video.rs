mod crtc;
mod field_data;
mod field_line;
mod video_crtc_registers_device;
mod video_memory_access;
mod video_registers;
mod video_ula_registers_device;

pub const MAX_LINES: usize = 320;

use std::{cell::RefCell, rc::Rc};

use crtc::Crtc;
pub use field_data::Field;
use field_line::FieldLine;
use video_crtc_registers_device::VideoCRTCRegistersDevice;
use video_memory_access::VideoMemoryAccess;
use video_registers::VideoRegisters;
use video_ula_registers_device::VideoULARegistersDevice;

#[cfg(test)]
pub use field_line::flags as field_line_flags;

#[derive(Default)]
pub struct Video {
    field_data: Field,
    crtc: Crtc,
    registers: Rc<RefCell<VideoRegisters>>,
    field_counter: u8,
    next_scanline_trigger: u64,
    vsync: bool,
}

impl Video {
    pub fn init(&mut self) {
        self.registers.borrow_mut().reset();

        self.crtc.init(&self.registers.borrow());

        self.field_counter = 0;
    }

    pub fn create_crtc_registers_device(&self) -> VideoCRTCRegistersDevice {
        VideoCRTCRegistersDevice::new(self.registers.clone())
    }

    pub fn create_ula_registers_device(&self) -> VideoULARegistersDevice {
        VideoULARegistersDevice::new(self.registers.clone())
    }

    pub fn process_scanline<'a>(
        &mut self,
        ic32_latch: u8,
        get_buffer: impl Fn(std::ops::Range<u16>) -> &'a [u8],
        mut on_vsync_change: impl FnMut(bool),
    ) -> bool {
        let registers = &self.registers.borrow();

        let snapshot_params = self.crtc.get_snapshot_params(registers);

        if snapshot_params.beam_scanline == 0 {
            self.field_counter = self.field_counter.wrapping_add(1);

            self.field_data.clear();
        }

        if snapshot_params.is_displayed {
            self.field_data.snapshot_scanline(
                snapshot_params.beam_scanline as usize,
                snapshot_params.address,
                snapshot_params.raster_address_even,
                snapshot_params.raster_address_odd,
                ic32_latch,
                self.field_counter,
                registers,
                get_buffer,
            );
        }

        self.crtc.advance_scanline(registers);

        self.next_scanline_trigger += self.crtc.get_next_scanline_cycles(registers);

        let new_vsync = self.crtc.is_in_vsync();
        if self.vsync != new_vsync {
            self.vsync = new_vsync;
            on_vsync_change(new_vsync);
        }

        // field is complete
        self.crtc.is_beam_reset()
    }

    pub fn get_next_scanline_trigger(&self) -> u64 {
        self.next_scanline_trigger
    }

    pub fn get_field_start(&self) -> *const Field {
        &self.field_data as *const Field
    }
}
