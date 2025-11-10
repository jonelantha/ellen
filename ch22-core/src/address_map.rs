mod io_space;
mod paged_rom;
mod ram;
mod rom;

use crate::devices::RomSelect;
use crate::interrupt_type::InterruptType;
use crate::system::Clock;
use crate::word::Word;
use io_space::IOSpace;
use paged_rom::PagedRom;
use ram::Ram;
use rom::Rom;

pub use io_space::{DeviceSpeed, IODeviceID};

pub struct AddressMap {
    pub ram: Ram,
    pub paged_rom: PagedRom,
    pub io_space: IOSpace,
    pub os_rom: Rom,
}

impl Default for AddressMap {
    fn default() -> Self {
        let mut io_space = IOSpace::default();
        let ram = Ram::default();
        let os_rom = Rom::new(0xc000);
        let paged_rom = PagedRom::new(0x8000);

        io_space.add_device(
            &[0xfe30, 0xfe31, 0xfe32, 0xfe33],
            Box::new(RomSelect::new(paged_rom.get_active_rom())),
            None,
            DeviceSpeed::TwoMhz,
        );

        AddressMap {
            ram,
            paged_rom,
            io_space,
            os_rom,
        }
    }
}

impl AddressMap {
    pub fn read(&mut self, address: Word, clock: &mut Clock) -> u8 {
        match address.1 {
            ..0x80 => self.ram.read(address),
            0x80..0xc0 => self.paged_rom.read(address),
            0xc0..0xfc => self.os_rom.read(address),
            0xfc..0xff => self.io_space.read(address, clock),
            0xff.. => self.os_rom.read(address),
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
