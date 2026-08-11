use std::cell::Cell;

use super::{address_map::AddressMap, clock::Clock, core::ROMS_LEN, cpu_bus::CpuBus};
use crate::address_spaces::{IOSpace, Ram, Rom};
use crate::cpu::Cpu;

pub struct Runner<'a, A: AddressMap> {
    pub cpu_bus: CpuBus<'a, A>,
    pub cpu: &'a mut Cpu,
}

impl<'a, A: AddressMap> Runner<'a, A> {
    pub fn new(
        clock: &'a mut Clock,
        ram: &'a mut Ram,
        roms: &'a [Rom; ROMS_LEN],
        io_space: &'a mut IOSpace,
        rom_select_latch: &'a Cell<usize>,
        address_map: A,
        cpu: &'a mut Cpu,
    ) -> Self {
        Self {
            cpu_bus: CpuBus::new(clock, ram, roms, io_space, rom_select_latch, address_map),
            cpu,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.cpu_bus);
    }

    pub fn run(&mut self, until: u64) {
        while self.cpu_bus.get_cycles() < until {
            self.cpu.handle_next_instruction(&mut self.cpu_bus);
        }
    }
}
