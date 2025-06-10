use std::{cell::Cell, rc::Rc};

use wasm_bindgen::prelude::*;

use super::device::*;

const ROM_SIZE: usize = 0x4000;

#[wasm_bindgen]
pub struct Ch22PagedRom {
    base_address: usize,
    roms: [[u8; ROM_SIZE]; 16],
    active_rom: Rc<Cell<usize>>,
}

#[wasm_bindgen]
impl Ch22PagedRom {
    pub fn new(base_address: usize) -> Self {
        Ch22PagedRom {
            base_address,
            roms: [[0; ROM_SIZE]; 16],
            active_rom: Rc::new(Cell::new(15)),
        }
    }

    pub fn get_rom_select(&self) -> Ch22RomSelect {
        Ch22RomSelect {
            active_rom: self.active_rom.clone(),
        }
    }

    pub fn set(&mut self, index: usize, data: &[u8]) {
        if data.len() != ROM_SIZE {
            panic!();
        }

        self.roms[index].copy_from_slice(data);
    }
}

impl Ch22Device for Ch22PagedRom {
    fn read(&mut self, address: u16, _cycles: u32) -> u8 {
        self.roms[self.active_rom.get()][address as usize - self.base_address]
    }

    fn is_slow(&self, _address: u16) -> bool {
        false
    }

    fn write(&mut self, _address: u16, _value: u8, _cycles: u32) -> bool {
        false
    }

    fn phase_2(&mut self, _address: u16, _cycles: u32) {}
}

////

#[wasm_bindgen]
pub struct Ch22RomSelect {
    active_rom: Rc<Cell<usize>>,
}

impl Ch22Device for Ch22RomSelect {
    fn read(&mut self, _address: u16, _cycles: u32) -> u8 {
        self.active_rom.get() as u8
    }

    fn is_slow(&self, _address: u16) -> bool {
        false
    }

    fn write(&mut self, _address: u16, value: u8, _cycles: u32) -> bool {
        self.active_rom.set(value as usize);

        false
    }

    fn phase_2(&mut self, _address: u16, _cycles: u32) {}
}
