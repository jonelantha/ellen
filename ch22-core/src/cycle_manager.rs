use crate::clock::Clock;
use crate::cpu_io::*;
use crate::devices_lib::address_map::*;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

#[derive(Default)]
pub struct CycleManager {
    pub clock: Clock,
    pub address_map: AddressMap,
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        self.address_map.read(address, &mut self.clock)
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        self.address_map.write(address, value, &mut self.clock);
    }

    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool {
        self.address_map.get_interrupt(interrupt_type, &self.clock)
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        self.address_map.phase_2(&self.clock);

        self.clock.inc();
    }
}
