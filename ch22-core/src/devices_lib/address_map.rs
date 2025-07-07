use crate::devices::io_space::*;
use crate::devices::paged_rom::*;
use crate::devices::ram::*;
use crate::devices::rom::*;
use crate::devices::rom_select::*;
use crate::devices_lib::addressable_device::*;
use crate::word::Word;

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
            false,
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
    pub fn get_device(&mut self, address: Word) -> &mut dyn AddressableDevice {
        match address.1 {
            ..0x80 => &mut self.ram,
            0x80..0xc0 => &mut self.paged_rom,
            0xc0..0xfc => &mut self.os_rom,
            0xfc..0xff => &mut self.io_space,
            0xff.. => &mut self.os_rom,
        }
    }
}
