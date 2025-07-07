use crate::cpu_io::*;
use crate::devices_lib::address_map::*;
use crate::devices_lib::timer_device_list::TimerDeviceList;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

pub struct CycleManager {
    pub cycles: u64,
    needs_phase_2: Option<Word>,
    pub address_map: AddressMap,
    pub timer_devices: TimerDeviceList,
}

impl CycleManager {
    pub fn new(address_map: AddressMap) -> Self {
        CycleManager {
            cycles: 0,
            needs_phase_2: None,
            address_map,
            timer_devices: TimerDeviceList::default(),
        }
    }
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        self.address_map
            .get_device(address)
            .read(address, &mut self.cycles)
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        let needs_phase_2 =
            self.address_map
                .get_device(address)
                .write(address, value, &mut self.cycles);

        if needs_phase_2 {
            self.needs_phase_2 = Some(address);
        }
    }

    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool {
        self.address_map
            .io_space
            .get_interrupt(interrupt_type, self.cycles)
    }

    fn instruction_ended(&mut self) {
        self.timer_devices.sync(self.cycles);
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        if let Some(address) = self.needs_phase_2 {
            let device = self.address_map.get_device(address);

            device.phase_2(address, self.cycles);

            self.needs_phase_2 = None;
        }

        self.cycles += 1;
    }
}
