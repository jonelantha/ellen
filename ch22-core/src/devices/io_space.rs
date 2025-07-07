use crate::clock::Clock;
use crate::devices_lib::addressable_device::AddressableDevice;
use crate::devices_lib::io_device_list::*;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

#[derive(Default)]
pub struct IOSpace {
    devices: IODeviceList,
}

impl IOSpace {
    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn AddressableDevice>,
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
    fn read(&mut self, address: Word, clock: &mut Clock) -> u8 {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return 0xff;
        };

        if config.slow {
            clock.slow_access(|clock| device.read(address, clock))
        } else {
            device.read(address, clock)
        }
    }

    fn write(&mut self, address: Word, value: u8, clock: &mut Clock) -> bool {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return false;
        };

        if config.slow {
            clock.slow_access(|clock| device.write(address, value, clock))
        } else {
            device.write(address, value, clock)
        }
    }

    fn phase_2(&mut self, address: Word, cycles: u64) {
        if let Some(device) = self.devices.get_by_address(address) {
            device.phase_2(address, cycles);
        }
    }
}
