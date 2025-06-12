use crate::word::Word;

use super::device::Ch22Device;

const RAM_SIZE: usize = 0x8000;

pub struct Ch22Ram {
    ram: [u8; RAM_SIZE],
}

impl Ch22Ram {
    pub fn new() -> Ch22Ram {
        Ch22Ram { ram: [0; RAM_SIZE] }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn ram_size(&self) -> usize {
        RAM_SIZE
    }
}

impl Ch22Device for Ch22Ram {
    fn read(&mut self, address: Word, _cycles: &mut u32) -> u8 {
        self.ram[Into::<usize>::into(address)]
    }

    fn write(&mut self, address: Word, value: u8, _cycles: &mut u32) -> bool {
        self.ram[Into::<usize>::into(address)] = value;

        false
    }

    fn phase_2(&mut self, _address: Word, _cycles: u32) {}
}
