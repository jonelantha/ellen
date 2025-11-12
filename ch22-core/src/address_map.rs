mod io_space;
mod paged_rom;
mod ram;
mod rom;

use crate::cpu::InterruptType;
use crate::system::Clock;
use crate::word::Word;

pub use io_space::IOSpace;
pub use paged_rom::PagedRom;
pub use ram::Ram;
pub use rom::Rom;

pub struct AddressMap<'a> {
    ram: &'a mut Ram,
    paged_rom: &'a mut PagedRom,
    io_space: &'a mut IOSpace,
    os_rom: &'a mut Rom,
}

impl<'a> AddressMap<'a> {
    pub fn new(
        ram: &'a mut Ram,
        paged_rom: &'a mut PagedRom,
        io_space: &'a mut IOSpace,
        os_rom: &'a mut Rom,
    ) -> Self {
        Self {
            ram,
            paged_rom,
            io_space,
            os_rom,
        }
    }

    pub fn read(&mut self, address: Word, clock: &mut Clock) -> u8 {
        match address.1 {
            ..0x80 => self.ram.read(address),
            0x80..0xc0 => self.paged_rom.read(address.rebased_to(0x80)),
            0xc0..0xfc => self.os_rom.read(address.rebased_to(0xc0)),
            0xfc..0xff => self.io_space.read(address, clock),
            0xff.. => self.os_rom.read(address.rebased_to(0xc0)),
        }
    }

    pub fn write(&mut self, address: Word, value: u8, clock: &mut Clock) {
        match address.1 {
            ..0x80 => self.ram.write(address, value),
            0x80..0xc0 => (), // paged rom
            0xc0..0xfc => (), // os rom
            0xfc..0xff => self.io_space.write(address, value, clock),
            0xff.. => (), // os rom
        }
    }

    pub fn phase_2(&mut self, clock: &Clock) {
        self.io_space.phase_2(clock);
    }

    pub fn get_interrupt(&mut self, interrupt_type: InterruptType, clock: &Clock) -> bool {
        self.io_space.get_interrupt(interrupt_type, clock)
    }
}
