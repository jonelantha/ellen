use core::panic;
use std::cell::RefCell;
use std::rc::Rc;

use crate::devices::IODevice;
use crate::video::VideoRegisters;
use crate::word::Word;

pub struct VideoCRTCRegistersDevice {
    video_crtc_registers: Rc<RefCell<VideoRegisters>>,
    control_reg: u8,
}

impl VideoCRTCRegistersDevice {
    pub fn new(video_crtc_registers: Rc<RefCell<VideoRegisters>>) -> Self {
        VideoCRTCRegistersDevice {
            video_crtc_registers,
            control_reg: 0,
        }
    }
}

impl IODevice for VideoCRTCRegistersDevice {
    fn read(&mut self, address: Word, _cycles: u64) -> u8 {
        if address.0 & 0x07 == 0x01 {
            let registers = self.video_crtc_registers.borrow_mut();

            match self.control_reg {
                12 | 13 => panic!("not impl"),

                14 => registers.crtc_r14_cursor_high,

                15 => registers.crtc_r15_cursor_low,

                16 | 17 => panic!("not impl"),
                _ => 0,
            }
        } else {
            0
        }
    }

    fn write(&mut self, address: Word, value: u8, _cycles: u64) -> bool {
        if address.0 & 0x07 == 0x01 {
            let mut registers = self.video_crtc_registers.borrow_mut();

            match self.control_reg {
                0 => registers.crtc_r0_horizontal_total = value,

                1 => registers.crtc_r1_horizontal_displayed = if value > 127 { 127 } else { value },

                2 => registers.crtc_r2_horizontal_sync_pos = value,

                3 => registers.crtc_r3_sync_width = value,

                4 => registers.crtc_r4_vertical_total = value,

                5 => registers.crtc_r5_vertical_total_adjust = value & 0x1f,

                6 => registers.crtc_r6_vertical_displayed = value,

                7 => registers.crtc_r7_vertical_sync_pos = value & 0x7f,

                8 => registers.crtc_r8_interlace_and_skew = value,

                9 => registers.crtc_r9_maximum_raster_address = value,

                10 => registers.crtc_r10_cursor_start_raster = value,

                11 => registers.crtc_r11_cursor_end_raster = value,

                12 => registers.crtc_r12_start_address_high = value,

                13 => registers.crtc_r13_start_address_low = value,

                14 => registers.crtc_r14_cursor_high = value & 0x3f,

                15 => registers.crtc_r15_cursor_low = value,

                _ => {}
            }
        } else {
            self.control_reg = value & 0x1f;
        }

        false
    }
}
