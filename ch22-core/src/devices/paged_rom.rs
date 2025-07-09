use std::cell::Cell;
use std::rc::Rc;

use crate::word::Word;

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

    pub fn get_active_rom(&self) -> Rc<Cell<usize>> {
        self.active_rom.clone()
    }

    pub fn load(&mut self, bank: u8, data: &[u8]) {
        if data.len() != ROM_SIZE {
            panic!();
        }

        self.roms[bank as usize].copy_from_slice(data);
    }

    pub fn read(&mut self, address: Word) -> u8 {
        self.roms[self.active_rom.get()][Into::<usize>::into(address) - self.base_address]
    }
}
