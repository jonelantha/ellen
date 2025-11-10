use crate::{
    address_map::AddressMap, clock::Clock, cpu::Cpu, cycle_manager::CycleManager,
    devices::timer_device_list::TimerDeviceList,
};

pub struct SystemRunner<'a> {
    cpu: &'a mut Cpu,
    cycle_manager: CycleManager<'a>,
}

impl<'a> SystemRunner<'a> {
    pub fn new(
        cycles: &'a mut u64,
        cpu: &'a mut Cpu,
        timer_devices: &'a mut TimerDeviceList,
        address_map: &'a mut AddressMap,
    ) -> Self {
        let clock = Clock::new(cycles, timer_devices);
        let cycle_manager = CycleManager::new(clock, address_map);

        Self { cpu, cycle_manager }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.cycle_manager);
    }

    pub fn run(&mut self, until: u64) {
        while self.cycle_manager.get_clock().get_cycles() < until {
            self.cpu.handle_next_instruction(&mut self.cycle_manager);
        }
    }
}
