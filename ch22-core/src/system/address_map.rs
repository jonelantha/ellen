use super::Clock;
use crate::address_spaces::{IOSpace, PagedRom, Ram, Rom};
use crate::word::Word;

pub trait AddressMap {
    fn read(
        &mut self,
        address: Word,
        clock: &mut Clock,
        ram: &mut Ram,
        paged_rom: &mut PagedRom,
        io_space: &mut IOSpace,
        os_rom: &mut Rom,
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
    FRead: FnMut(Word, &mut Clock, &mut Ram, &mut PagedRom, &mut IOSpace, &mut Rom) -> u8,
    FWrite: FnMut(Word, u8, &mut Clock, &mut Ram, &mut IOSpace),
{
    pub read: FRead,
    pub write: FWrite,
}

impl<FRead, FWrite> AddressMap for FnAddressMap<FRead, FWrite>
where
    FRead: FnMut(Word, &mut Clock, &mut Ram, &mut PagedRom, &mut IOSpace, &mut Rom) -> u8,
    FWrite: FnMut(Word, u8, &mut Clock, &mut Ram, &mut IOSpace),
{
    fn read(
        &mut self,
        address: Word,
        clock: &mut Clock,
        ram: &mut Ram,
        paged_rom: &mut PagedRom,
        io_space: &mut IOSpace,
        os_rom: &mut Rom,
    ) -> u8 {
        (self.read)(address, clock, ram, paged_rom, io_space, os_rom)
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
