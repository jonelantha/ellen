use wasm_bindgen::prelude::*;

use super::device::*;

const ROM_SIZE: usize = 0x4000;

#[wasm_bindgen]
pub struct Ch22Rom {
    base_address: usize,
    rom: [u8; ROM_SIZE],
}

#[wasm_bindgen]
impl Ch22Rom {
    pub fn new(base_address: usize) -> Self {
        Ch22Rom {
            base_address,
            rom: [0; ROM_SIZE],
        }
    }

    pub fn set(&mut self, data: &[u8]) {
        if data.len() != ROM_SIZE {
            panic!();
        }

        self.rom.copy_from_slice(data);
    }
}

impl Ch22Device for Ch22Rom {
    fn read(&mut self, address: u16, _cycles: u32) -> u8 {
        self.rom[address as usize - self.base_address]
    }

    fn is_slow(&self, _address: u16) -> bool {
        false
    }

    fn write(&mut self, _address: u16, _value: u8, _cycles: u32) -> bool {
        false
    }

    fn phase_2(&mut self, _address: u16, _cycles: u32) {}
}
