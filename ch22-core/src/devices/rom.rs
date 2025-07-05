use crate::word::Word;

use super::device::*;

const ROM_SIZE: usize = 0x4000;

pub struct Ch22Rom {
    base_address: usize,
    rom: [u8; ROM_SIZE],
}

impl Ch22Rom {
    pub fn new(base_address: usize) -> Self {
        Ch22Rom {
            base_address,
            rom: [0; ROM_SIZE],
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        if data.len() != ROM_SIZE {
            panic!();
        }

        self.rom.copy_from_slice(data);
    }
}

impl Ch22Device for Ch22Rom {
    fn read(&mut self, address: Word, _cycles: &mut u64) -> u8 {
        self.rom[Into::<usize>::into(address) - self.base_address]
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: &mut u64) -> bool {
        false
    }
}
