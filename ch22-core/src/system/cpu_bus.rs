use std::cell::Cell;

use super::{address_map::AddressMap, clock::Clock, core::ROMS_LEN};
use crate::address_spaces::{IOSpace, Ram, Rom};
use crate::cpu::{CpuIO, InterruptType};
use crate::word::Word;

pub struct CpuBus<'a, A: AddressMap> {
    clock: Clock<'a>,
    ram: &'a mut Ram,
    roms: &'a [Rom; ROMS_LEN],
    io_space: &'a mut IOSpace,
    rom_select_latch: &'a Cell<usize>,
    address_map: A,
}

impl<'a, A: AddressMap> CpuBus<'a, A> {
    pub fn new(
        clock: Clock<'a>,
        ram: &'a mut Ram,
        roms: &'a [Rom; ROMS_LEN],
        io_space: &'a mut IOSpace,
        rom_select_latch: &'a Cell<usize>,
        address_map: A,
    ) -> Self {
        Self {
            clock,
            ram,
            roms,
            io_space,
            rom_select_latch,
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
            self.roms,
            self.io_space,
            self.rom_select_latch,
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
