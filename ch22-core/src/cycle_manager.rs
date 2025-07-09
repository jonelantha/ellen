use crate::clock::Clock;
use crate::cpu_io::*;
use crate::devices_lib::address_map::*;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

#[derive(Default)]
pub struct CycleManager {
    pub clock: Clock,
    needs_phase_2: Option<Word>,
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

        let needs_phase_2 = self.address_map.write(address, value, &mut self.clock);

        if needs_phase_2 {
            self.needs_phase_2 = Some(address);
        }
    }

    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool {
        self.address_map
            .get_interrupt(interrupt_type, &mut self.clock)
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        if let Some(address) = self.needs_phase_2 {
            self.address_map.phase_2(address, &mut self.clock);

            self.needs_phase_2 = None;
        }

        self.clock.inc();
    }
}
