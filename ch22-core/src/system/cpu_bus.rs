use crate::address_map::AddressMap;
use crate::cpu::CpuIO;
use crate::interrupt_type::InterruptType;
use crate::system::clock::Clock;
use crate::word::Word;

pub struct CpuBus<'a> {
    clock: Clock<'a>,
    address_map: &'a mut AddressMap,
}

impl<'a> CpuBus<'a> {
    pub fn new(clock: Clock<'a>, address_map: &'a mut AddressMap) -> Self {
        Self { clock, address_map }
    }
}

impl CpuIO for CpuBus<'_> {
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

impl CpuBus<'_> {
    fn end_previous_cycle(&mut self) {
        self.address_map.phase_2(&self.clock);

        self.clock.inc();
    }

    pub fn get_clock(&'_ self) -> &'_ Clock<'_> {
        &self.clock
    }
}
