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
            match self.control_reg {
                12 | 13 => panic!("not impl"),
                14 => self.video_crtc_registers.borrow().crtc_registers[14],
                15 => self.video_crtc_registers.borrow().crtc_registers[15],
                16 | 17 => panic!("not impl"),
                _ => 0,
            }
        } else {
            0
        }
    }

    fn write(&mut self, address: Word, value: u8, _cycles: u64) -> bool {
        if address.0 & 0x07 == 0x01 {
            let reg_index = self.control_reg as usize;

            if let Some(value) = Self::mask_register_value(reg_index, value) {
                self.video_crtc_registers.borrow_mut().crtc_registers[reg_index] = value;
            }
        } else {
            self.control_reg = value & 0x1f;
        }

        false
    }
}

impl VideoCRTCRegistersDevice {
    fn mask_register_value(reg_index: usize, value: u8) -> Option<u8> {
        match reg_index {
            0 => Some(value),
            1 => Some(if value > 127 { 127 } else { value }),
            2..=4 => Some(value),
            5 => Some(value & 0x1f),
            6 => Some(value),
            7 => Some(value & 0x7f),
            8..=13 => Some(value),
            14 => Some(value & 0x3f),
            15 => Some(value),
            _ => None,
        }
    }
}
