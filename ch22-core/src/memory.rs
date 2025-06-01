const RAM_SIZE: usize = 0x10000;

pub struct Ch22Memory {
    ram: [u8; RAM_SIZE],
    is_io: Box<dyn Fn(u16) -> bool>,
    read_fallback: Box<dyn Fn(u16, u32) -> u8>,
    write_fallback: Box<dyn Fn(u16, u8, u32) -> bool>,
}

impl Ch22Memory {
    pub fn new(
        is_io: Box<dyn Fn(u16) -> bool>,
        read_fallback: Box<dyn Fn(u16, u32) -> u8>,
        write_fallback: Box<dyn Fn(u16, u8, u32) -> bool>,
    ) -> Ch22Memory {
        Ch22Memory {
            ram: [0; RAM_SIZE],
            is_io,
            read_fallback,
            write_fallback,
        }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn ram_size(&self) -> usize {
        RAM_SIZE
    }

    pub fn read(&self, address: u16, machine_cycles: u32) -> u8 {
        match address {
            ..0x8000 => self.ram[address as usize],
            0x8000..0xc000 => (self.read_fallback)(address, machine_cycles),
            0xc000..0xfc00 => self.ram[address as usize],
            0xfc00..0xff00 => (self.read_fallback)(address, machine_cycles),
            0xff00.. => self.ram[address as usize],
        }
    }

    pub fn is_io(&self, address: u16) -> bool {
        match address {
            0xfc00..0xff00 => (self.is_io)(address),
            _ => false,
        }
    }

    pub fn write(&mut self, address: u16, value: u8, machine_cycles: u32) -> bool {
        match address {
            ..0x8000 => {
                self.ram[address as usize] = value;
                false
            }
            0x8000..0xc000 => false,
            _ => (self.write_fallback)(address, value, machine_cycles),
        }
    }
}
