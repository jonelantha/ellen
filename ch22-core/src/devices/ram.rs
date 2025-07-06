use crate::word::Word;

use super::addressable_device::AddressableDevice;

const RAM_SIZE: usize = 0x8000;

pub struct Ram {
    ram: [u8; RAM_SIZE],
}

impl Ram {
    pub fn new() -> Ram {
        Ram { ram: [0; RAM_SIZE] }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn ram_size(&self) -> usize {
        RAM_SIZE
    }
}

impl AddressableDevice for Ram {
    fn read(&mut self, address: Word, _cycles: &mut u64) -> u8 {
        self.ram[Into::<usize>::into(address)]
    }

    fn write(&mut self, address: Word, value: u8, _cycles: &mut u64) -> bool {
        self.ram[Into::<usize>::into(address)] = value;

        false
    }
}
