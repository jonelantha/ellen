use crate::clock::Clock;
use crate::devices_lib::io_device::IODevice;
use crate::devices_lib::io_device_list::*;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

#[derive(Default)]
pub struct IOSpace {
    devices: IODeviceList,
    phase_2_data: Option<(Word, u8)>,
}

impl IOSpace {
    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        one_mhz: bool,
    ) -> IODeviceID {
        self.devices
            .add_device(addresses, device, interrupt_type, one_mhz)
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

        let value;

        if config.one_mhz {
            clock.one_mhz_sync();

            value = device.read(address, clock.get_cycles());

            clock.inc();
        } else {
            value = device.read(address, clock.get_cycles())
        }

        value
    }

    pub fn write(&mut self, address: Word, value: u8, clock: &mut Clock) {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return;
        };

        let needs_phase_2;

        if config.one_mhz {
            clock.one_mhz_sync();

            needs_phase_2 = device.write(address, value, clock.get_cycles());

            clock.inc();
        } else {
            needs_phase_2 = device.write(address, value, clock.get_cycles());
        };

        if needs_phase_2 {
            self.phase_2_data = Some((address, value));
        }
    }

    pub fn phase_2(&mut self, clock: &mut Clock) {
        if let Some((address, value)) = self.phase_2_data {
            if let Some(device) = self.devices.get_by_address(address) {
                device.phase_2(address, value, clock.get_cycles());
            }

            self.phase_2_data = None;
        }
    }
}
