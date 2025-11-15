use super::{address_map::AddressMap, cpu_bus::CpuBus};
use crate::cpu::Cpu;

pub struct SystemRunner<'a, A: AddressMap> {
    cpu_bus: CpuBus<'a, A>,
    cpu: &'a mut Cpu,
}

pub trait SystemRunnerTrait {
    fn reset(&mut self);
    fn run(&mut self, until: u64);
}

impl<'a, A: AddressMap> SystemRunner<'a, A> {
    pub fn new(cpu_bus: CpuBus<'a, A>, cpu: &'a mut Cpu) -> Self {
        SystemRunner { cpu_bus, cpu }
    }
}

impl<'a, A: AddressMap> SystemRunnerTrait for SystemRunner<'a, A> {
    fn reset(&mut self) {
        self.cpu.reset(&mut self.cpu_bus);
    }

    fn run(&mut self, until: u64) {
        while self.cpu_bus.get_cycles() < until {
            self.cpu.handle_next_instruction(&mut self.cpu_bus);
        }
    }
}
