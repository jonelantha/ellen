use super::{clock::Clock, cpu_bus::CpuBus};
use crate::{address_map::AddressMap, cpu::Cpu, devices::TimerDeviceList};

pub struct SystemRunner<'a> {
    cpu: &'a mut Cpu,
    cpu_bus: CpuBus<'a>,
}

impl<'a> SystemRunner<'a> {
    pub fn new(
        cycles: &'a mut u64,
        cpu: &'a mut Cpu,
        timer_devices: &'a mut TimerDeviceList,
        address_map: &'a mut AddressMap,
    ) -> Self {
        let clock = Clock::new(cycles, timer_devices);
        let cpu_bus = CpuBus::new(clock, address_map);

        Self { cpu, cpu_bus }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.cpu_bus);
    }

    pub fn run(&mut self, until: u64) {
        while self.cpu_bus.get_clock().get_cycles() < until {
            self.cpu.handle_next_instruction(&mut self.cpu_bus);
        }
    }
}
