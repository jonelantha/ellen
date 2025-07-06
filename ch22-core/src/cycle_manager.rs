use crate::cpu_io::*;
use crate::device_map::*;
use crate::devices::io_space::DeviceID;
use crate::devices::syncable_device::SyncableDevice;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

pub struct CycleManager {
    pub cycles: u64,
    needs_phase_2: Option<Word>,
    pub device_map: DeviceMap,
    pub run_until: u64,
    sync_devices: Vec<Box<dyn SyncableDevice>>,
}

impl CycleManager {
    pub fn new(device_map: DeviceMap) -> Self {
        CycleManager {
            cycles: 0,
            needs_phase_2: None,
            device_map,
            run_until: 0,
            sync_devices: Vec::new(),
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

    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool {
        self.device_map
            .io_space
            .get_interrupt(interrupt_type, self.cycles)
    }

    fn instruction_ended(&mut self) {
        self.sync();
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
    }

    pub fn sync(&mut self) {
        for device in self.sync_devices.iter_mut() {
            device.sync(self.cycles);
        }
    }

    pub fn add_device(&mut self, device: Box<dyn SyncableDevice>) -> DeviceID {
        self.sync_devices.push(device);

        // assumes devices will not be removed
        let device_id = self.sync_devices.len() - 1;

        device_id
    }

    pub fn set_device_trigger(&mut self, device_id: DeviceID, trigger: Option<u64>) {
        self.sync_devices[device_id].set_trigger(trigger);
    }
}
