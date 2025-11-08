use crate::address_map::*;
use crate::clock::Clock;
use crate::cpu::cpu_io::*;
use crate::devices::timer_device_list::TimerDeviceList;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

pub struct CycleManager<'a> {
    clock: Clock<'a>,
    address_map: &'a mut AddressMap,
}

impl<'a> CycleManager<'a> {
    pub fn new(
        cycles: &'a mut u64,
        timer_devices: &'a mut TimerDeviceList,
        address_map: &'a mut AddressMap,
    ) -> Self {
        Self {
            clock: Clock::new(cycles, timer_devices),
            address_map,
        }
    }
}

impl CpuIO for CycleManager<'_> {
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

impl CycleManager<'_> {
    fn end_previous_cycle(&mut self) {
        self.address_map.phase_2(&self.clock);

        self.clock.inc();
    }

    pub fn repeat<F>(&mut self, run_until: u64, mut f: F) -> u64
    where
        F: FnMut(&mut Self) -> (),
    {
        while self.clock.get_cycles() < run_until {
            f(self);
        }

        self.clock.get_cycles()
    }
}
