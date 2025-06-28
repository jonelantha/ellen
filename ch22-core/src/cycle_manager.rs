use crate::cpu_io::*;
use crate::device_map::*;
use crate::word::Word;

const CYCLE_WRAP: u32 = 0x3FFFFFFF;

pub struct CycleManager {
    pub cycles: u32,
    needs_phase_2: Option<Word>,
    pub device_map: DeviceMap,
    pub run_until: u32,
}

impl CycleManager {
    pub fn new(device_map: DeviceMap) -> Self {
        CycleManager {
            cycles: 0,
            needs_phase_2: None,
            device_map,
            run_until: 0,
        }
    }
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        self.device_map
            .get_device(address)
            .read(address, &mut self.cycles)
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        let needs_phase_2 =
            self.device_map
                .get_device(address)
                .write(address, value, &mut self.cycles);

        if needs_phase_2 {
            self.needs_phase_2 = Some(address);
        }
    }

    fn get_irq(&mut self) -> bool {
        self.device_map.io_space.get_irq(self.cycles)
    }

    fn get_nmi(&mut self) -> bool {
        self.device_map.io_space.get_nmi(self.cycles)
    }

    fn instruction_ended(&mut self) {
        self.device_map.io_space.sync(self.cycles)
    }
}

impl CycleManager {
    pub fn is_running(&self) -> bool {
        self.cycles < self.run_until
    }

    fn end_previous_cycle(&mut self) {
        if let Some(address) = self.needs_phase_2 {
            let device = self.device_map.get_device(address);

            device.phase_2(address, self.cycles);

            self.needs_phase_2 = None;
        }

        self.cycles += 1;

        if self.cycles > CYCLE_WRAP {
            self.device_map.io_space.wrap_triggers(CYCLE_WRAP);
            self.cycles -= CYCLE_WRAP;
            self.run_until -= CYCLE_WRAP;
        }
    }
}
