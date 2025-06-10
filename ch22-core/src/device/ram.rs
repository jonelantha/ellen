use wasm_bindgen::prelude::*;

use super::device::Ch22Device;

const RAM_SIZE: usize = 0x8000;

#[wasm_bindgen]
pub struct Ch22Ram {
    ram: [u8; RAM_SIZE],
}

#[wasm_bindgen]
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
    fn read(&mut self, address: u16, _cycles: u32) -> u8 {
        self.ram[address as usize]
    }

    fn is_slow(&self, _address: u16) -> bool {
        false
    }

    fn write(&mut self, address: u16, value: u8, _cycles: u32) -> bool {
        self.ram[address as usize] = value;

        false
    }

    fn phase_2(&mut self, _address: u16, _cycles: u32) {}
}
