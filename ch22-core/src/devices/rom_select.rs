use std::cell::Cell;
use std::rc::Rc;

use super::io_device::*;
use crate::word::Word;

pub struct RomSelect {
    active_rom: Rc<Cell<usize>>,
}

impl RomSelect {
    pub fn new(active_rom: Rc<Cell<usize>>) -> Self {
        RomSelect { active_rom }
    }
}

impl IODevice for RomSelect {
    fn read(&mut self, _address: Word, _cycles: u64) -> u8 {
        self.active_rom.get() as u8
    }

    fn write(&mut self, _address: Word, value: u8, _cycles: u64) -> bool {
        self.active_rom.set((value & 0x0f) as usize); // 4 bit latch

        false
    }
}
