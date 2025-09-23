use crate::word::Word;
use std::ops::Range;

const RAM_SIZE: usize = 0x8000;

pub struct Ram {
    ram: [u8; RAM_SIZE],
}

impl Default for Ram {
    fn default() -> Ram {
        Ram { ram: [0; RAM_SIZE] }
    }
}

impl Ram {
    pub fn ram_start(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn ram_size(&self) -> usize {
        RAM_SIZE
    }

    pub fn read(&mut self, address: Word) -> u8 {
        self.ram[Into::<usize>::into(address)]
    }

    pub fn write(&mut self, address: Word, value: u8) {
        self.ram[Into::<usize>::into(address)] = value;
    }

    pub fn slice(&self, range: Range<u16>) -> &[u8] {
        &self.ram[range.start as usize..range.end as usize]
    }
}
