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

    pub fn read(&mut self, address: Word, clock: &mut Clock) -> u8 {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return 0xff;
        };

        if config.slow {
            clock.one_mhz_sync();

            let value = device.read(address, clock.get_cycles());

            clock.inc();

            value
        } else {
            device.read(address, clock.get_cycles())
        }
    }

    pub fn write(&mut self, address: Word, value: u8, clock: &mut Clock) -> bool {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return false;
        };

        if config.slow {
            clock.one_mhz_sync();

            let value = device.write(address, value, clock.get_cycles());

            clock.inc();

            value
        } else {
            device.write(address, value, clock.get_cycles())
        }
    }

    pub fn phase_2(&mut self, address: Word, clock: &mut Clock) {
        if let Some(device) = self.devices.get_by_address(address) {
            device.phase_2(address, clock.get_cycles());
        }
    }
}
