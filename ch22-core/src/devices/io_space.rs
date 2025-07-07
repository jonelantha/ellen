use crate::devices_lib::addressable_device::AddressableDevice;
use crate::devices_lib::io_device::IODevice;
use crate::devices_lib::io_device_list::{IODeviceID, IODeviceList};
use crate::interrupt_type::InterruptType;
use crate::word::Word;

pub struct IOSpace {
    devices: IODeviceList,
}

impl IOSpace {
    pub fn new() -> Self {
        IOSpace {
            devices: IODeviceList::default(),
        }
    }

    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        slow: bool,
    ) -> IODeviceID {
        self.devices
            .add_device(addresses, device, interrupt_type, slow)
    }

    pub fn get_interrupt(&mut self, interrupt_type: InterruptType, cycles: u64) -> bool {
        self.devices
            .get_by_interrupt_type(interrupt_type)
            .any(|device| device.get_interrupt(cycles))
    }

    pub fn set_interrupt(&mut self, device_id: IODeviceID, iterrupt: bool) {
        self.devices.get_by_id(device_id).set_interrupt(iterrupt);
    }
}

impl AddressableDevice for IOSpace {
    fn read(&mut self, address: Word, cycles: &mut u64) -> u8 {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return 0xff;
        };

        if config.slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let value = device.read(address, *cycles);

        if config.slow {
            *cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8, cycles: &mut u64) -> bool {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return false;
        };

        if config.slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let needs_phase_2 = device.write(address, value, *cycles);

        if config.slow {
            *cycles += 1;
        }

        needs_phase_2
    }

    fn phase_2(&mut self, address: Word, cycles: u64) {
        if let Some(device) = self.devices.get_by_address(address) {
            device.phase_2(cycles);
        }
    }
}
