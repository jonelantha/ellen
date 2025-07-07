use crate::clock::Clock;
use crate::word::Word;

use crate::devices_lib::addressable_device::AddressableDevice;

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
}

impl AddressableDevice for Ram {
    fn read(&mut self, address: Word, _clock: &mut Clock) -> u8 {
        self.ram[Into::<usize>::into(address)]
    }

    fn write(&mut self, address: Word, value: u8, _clock: &mut Clock) -> bool {
        self.ram[Into::<usize>::into(address)] = value;

        false
    }
}
