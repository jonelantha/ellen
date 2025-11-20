use std::cell::RefCell;
use std::rc::Rc;

use crate::devices::IODevice;
use crate::video::VideoRegisters;
use crate::word::Word;

pub struct VideoULARegistersDevice {
    video_registers: Rc<RefCell<VideoRegisters>>,
}

impl VideoULARegistersDevice {
    pub fn new(video_registers: Rc<RefCell<VideoRegisters>>) -> Self {
        VideoULARegistersDevice { video_registers }
    }
}

impl IODevice for VideoULARegistersDevice {
    fn read(&mut self, _address: Word, _cycles: u64) -> u8 {
        0xfe
    }

    fn write(&mut self, address: Word, value: u8, _cycles: u64) -> bool {
        if address.0 & 0x03 == 0x01 {
            let entry = (value & 0xf0) >> 4;
            let value = (value & 0x0f) ^ 7;

            self.video_registers
                .borrow_mut()
                .set_ula_palette(entry, value);
        } else {
            self.video_registers.borrow_mut().ula_control = value;
        }

        false
    }
}
