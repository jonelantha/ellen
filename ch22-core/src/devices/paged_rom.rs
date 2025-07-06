use std::{cell::Cell, rc::Rc};

use crate::devices::io_device::IODevice;
use crate::word::Word;

use super::addressable_device::*;

const ROM_SIZE: usize = 0x4000;

pub struct PagedRom {
    base_address: usize,
    roms: [[u8; ROM_SIZE]; 16],
    active_rom: Rc<Cell<usize>>,
}

impl PagedRom {
    pub fn new(base_address: usize) -> Self {
        PagedRom {
            base_address,
            roms: [[0; ROM_SIZE]; 16],
            active_rom: Rc::new(Cell::new(15)),
        }
    }

    pub fn get_rom_select(&self) -> RomSelect {
        RomSelect {
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

impl AddressableDevice for PagedRom {
    fn read(&mut self, address: Word, _cycles: &mut u64) -> u8 {
        self.roms[self.active_rom.get()][Into::<usize>::into(address) - self.base_address]
    }
}

pub struct RomSelect {
    active_rom: Rc<Cell<usize>>,
}

impl IODevice for RomSelect {
    fn read(&mut self, _address: Word, _cycles: u64) -> u8 {
        self.active_rom.get() as u8
    }

    fn write(&mut self, _address: Word, value: u8, _cycles: u64) -> bool {
        self.active_rom.set(value as usize);

        false
    }
}
