use std::{cell::Cell, rc::Rc};

use crate::{devices::io_device::Ch22IODevice, word::Word};

use super::device::*;

const ROM_SIZE: usize = 0x4000;

pub struct Ch22PagedRom {
    base_address: usize,
    roms: [[u8; ROM_SIZE]; 16],
    active_rom: Rc<Cell<usize>>,
}

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

    pub fn load(&mut self, bank: u8, data: &[u8]) {
        if data.len() != ROM_SIZE {
            panic!();
        }

        self.roms[bank as usize].copy_from_slice(data);
    }
}

impl Ch22Device for Ch22PagedRom {
    fn read(&mut self, address: Word, _cycles: &mut u32) -> u8 {
        self.roms[self.active_rom.get()][Into::<usize>::into(address) - self.base_address]
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: &mut u32) -> bool {
        false
    }

    fn phase_2(&mut self, _address: Word, _cycles: u32) {}
}

pub struct Ch22RomSelect {
    active_rom: Rc<Cell<usize>>,
}

impl Ch22IODevice for Ch22RomSelect {
    fn read(&mut self, _address: Word, _cycles: u32, _interrupt: &mut u8) -> u8 {
        self.active_rom.get() as u8
    }

    fn is_slow(&self) -> bool {
        false
    }

    fn write(&mut self, _address: Word, value: u8, _cycles: u32, _interrupt: &mut u8) -> bool {
        self.active_rom.set(value as usize);

        false
    }
}
