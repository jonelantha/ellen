use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::cpu_io::*;
use crate::device::*;
use crate::memory::*;
use crate::word::Word;

const CYCLE_WRAP: u32 = 0x3FFFFFFF;

pub struct CycleManager {
    pub machine_cycles: u32,
    needs_phase_2: Option<u16>,
    get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
    wrap_counts: Box<dyn Fn(u32)>,
    pub device_list: DeviceList,
}

impl CycleManager {
    pub fn new(
        memory: Ch22Memory,
        get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
        wrap_counts: Box<dyn Fn(u32)>,
    ) -> Self {
        CycleManager {
            machine_cycles: 0,
            get_irq_nmi,
            wrap_counts,
            needs_phase_2: None,
            device_list: DeviceList::new(memory),
        }
    }
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        let address: u16 = address.into();

        let device = self.device_list.get_device(address);

        let is_slow = device.is_slow(address);

        if is_slow && self.machine_cycles & 1 != 0 {
            self.machine_cycles += 1;
        }

        let value = device.read(address, self.machine_cycles);

        if is_slow {
            self.machine_cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        let address: u16 = address.into();

        let device = self.device_list.get_device(address);

        let is_slow = device.is_slow(address);

        if is_slow && self.machine_cycles & 1 != 0 {
            self.machine_cycles += 1;
        }

        if device.write(address, value, self.machine_cycles) {
            self.needs_phase_2 = Some(address);
        }

        if is_slow {
            self.machine_cycles += 1;
        }
    }

    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool) {
        let (irq, nmi) = (self.get_irq_nmi)(self.machine_cycles);

        (irq & !interrupt_disable, nmi)
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        if let Some(address) = self.needs_phase_2 {
            let device = self.device_list.get_device(address);

            device.phase_2(self.machine_cycles);

            self.needs_phase_2 = None;
        }

        self.machine_cycles += 1;

        if self.machine_cycles > CYCLE_WRAP {
            (self.wrap_counts)(CYCLE_WRAP);
            self.machine_cycles -= CYCLE_WRAP;
        }
    }
}

pub struct DeviceList {
    next_device_id: u8,
    devices: HashMap<u8, Box<dyn Ch22Device>>,
    address_to_device_id: HashMap<u16, u8>,
    pub memory: Ch22Memory,
}

impl DeviceList {
    pub fn new(memory: Ch22Memory) -> Self {
        DeviceList {
            next_device_id: 0,
            devices: HashMap::new(),
            address_to_device_id: HashMap::new(),
            memory,
        }
    }

    pub fn add_device(&mut self, addresses: RangeInclusive<u16>, device: Box<dyn Ch22Device>) {
        let device_id = self.next_device_id;

        self.devices.insert(device_id, device);

        for address in addresses {
            self.address_to_device_id.insert(address, device_id);
        }

        self.next_device_id += 1;
    }

    fn get_device(&mut self, address: u16) -> &mut dyn Ch22Device {
        let Some(device_id) = self.address_to_device_id.get(&address) else {
            return &mut self.memory;
        };

        self.devices.get_mut(device_id).unwrap().as_mut()
    }
}
