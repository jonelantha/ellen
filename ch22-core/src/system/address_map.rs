use std::cell::Cell;

use super::{clock::Clock, core::ROMS_LEN};
use crate::address_spaces::{IOSpace, Ram, Rom};
use crate::word::Word;

pub trait AddressMap {
    fn read(
        &mut self,
        address: Word,
        clock: &mut Clock,
        ram: &mut Ram,
        roms: &[Rom; ROMS_LEN],
        io_space: &mut IOSpace,
        rom_select_latch: &Cell<usize>,
    ) -> u8;

    fn write(
        &mut self,
        address: Word,
        value: u8,
        clock: &mut Clock,
        ram: &mut Ram,
        io_space: &mut IOSpace,
    );
}

pub struct FnAddressMap<FRead, FWrite>
where
    FRead: FnMut(Word, &mut Clock, &mut Ram, &[Rom; ROMS_LEN], &mut IOSpace, &Cell<usize>) -> u8,
    FWrite: FnMut(Word, u8, &mut Clock, &mut Ram, &mut IOSpace),
{
    pub read: FRead,
    pub write: FWrite,
}

impl<FRead, FWrite> AddressMap for FnAddressMap<FRead, FWrite>
where
    FRead: FnMut(Word, &mut Clock, &mut Ram, &[Rom; ROMS_LEN], &mut IOSpace, &Cell<usize>) -> u8,
    FWrite: FnMut(Word, u8, &mut Clock, &mut Ram, &mut IOSpace),
{
    fn read(
        &mut self,
        address: Word,
        clock: &mut Clock,
        ram: &mut Ram,
        roms: &[Rom; ROMS_LEN],
        io_space: &mut IOSpace,
        rom_select_latch: &Cell<usize>,
    ) -> u8 {
        (self.read)(address, clock, ram, roms, io_space, rom_select_latch)
    }

    fn write(
        &mut self,
        address: Word,
        value: u8,
        clock: &mut Clock,
        ram: &mut Ram,
        io_space: &mut IOSpace,
    ) {
        (self.write)(address, value, clock, ram, io_space)
    }
}
