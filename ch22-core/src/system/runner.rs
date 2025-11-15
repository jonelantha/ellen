use super::{address_map::AddressMap, cpu_bus::CpuBus};
use crate::cpu::Cpu;

pub struct Runner<'a, A: AddressMap> {
    pub cpu_bus: CpuBus<'a, A>,
    pub cpu: &'a mut Cpu,
}

pub trait RunnerTrait {
    fn reset(&mut self);
    fn run(&mut self, until: u64);
}

impl<'a, A: AddressMap> RunnerTrait for Runner<'a, A> {
    fn reset(&mut self) {
        self.cpu.reset(&mut self.cpu_bus);
    }

    fn run(&mut self, until: u64) {
        while self.cpu_bus.get_cycles() < until {
            self.cpu.handle_next_instruction(&mut self.cpu_bus);
        }
    }
}
