use crate::device::Ch22Device;

const RAM_SIZE: usize = 0x10000;

pub struct Ch22Memory {
    ram: [u8; RAM_SIZE],
    read_fallback: Box<dyn Fn(u16, u32) -> u8>,
}

impl Ch22Memory {
    pub fn new(read_fallback: Box<dyn Fn(u16, u32) -> u8>) -> Ch22Memory {
        Ch22Memory {
            ram: [0; RAM_SIZE],
            read_fallback,
        }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn ram_size(&self) -> usize {
        RAM_SIZE
    }
}

impl Ch22Device for Ch22Memory {
    fn read(&mut self, address: u16, cycles: u32) -> u8 {
        match address {
            ..0x8000 => self.ram[address as usize],
            0x8000..0xc000 => (self.read_fallback)(address, cycles),
            0xc000..0xfc00 => self.ram[address as usize],
            0xfc00..0xff00 => (self.read_fallback)(address, cycles),
            0xff00.. => self.ram[address as usize],
        }
    }

    fn is_slow(&self, _address: u16) -> bool {
        false
    }

    fn write(&mut self, address: u16, value: u8, _cycles: u32) -> bool {
        if let ..0x8000 = address {
            self.ram[address as usize] = value
        }

        false
    }

    fn phase_2(&mut self, _cycles: u32) {}
}
