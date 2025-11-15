use super::{address_map::AddressMap, clock::Clock};
use crate::address_spaces::{IOSpace, PagedRom, Ram, Rom};
use crate::cpu::{CpuIO, InterruptType};
use crate::word::Word;

pub struct CpuBus<'a, A: AddressMap> {
    clock: Clock<'a>,
    ram: &'a mut Ram,
    paged_rom: &'a mut PagedRom,
    io_space: &'a mut IOSpace,
    os_rom: &'a mut Rom,
    address_map: A,
}

impl<'a, A: AddressMap> CpuBus<'a, A> {
    pub fn new(
        clock: Clock<'a>,
        ram: &'a mut Ram,
        paged_rom: &'a mut PagedRom,
        io_space: &'a mut IOSpace,
        os_rom: &'a mut Rom,
        address_map: A,
    ) -> Self {
        Self {
            clock,
            ram,
            paged_rom,
            io_space,
            os_rom,
            address_map,
        }
    }
}

impl<A: AddressMap> CpuIO for CpuBus<'_, A> {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        self.address_map.read(
            address,
            &mut self.clock,
            self.ram,
            self.paged_rom,
            self.io_space,
            self.os_rom,
        )
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        self.address_map
            .write(address, value, &mut self.clock, self.ram, self.io_space);
    }

    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool {
        self.io_space.get_interrupt(interrupt_type, &self.clock)
    }
}

impl<A: AddressMap> CpuBus<'_, A> {
    fn end_previous_cycle(&mut self) {
        self.io_space.phase_2(&self.clock);

        self.clock.inc();
    }

    pub fn get_cycles(&self) -> u64 {
        self.clock.get_cycles()
    }
}
