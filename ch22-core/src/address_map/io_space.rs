#[cfg(test)]
mod tests;

use crate::cpu::InterruptType;
use crate::devices::{DeviceSpeed, IODevice, IODeviceID, IODeviceList};
use crate::system::Clock;
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
        speed: DeviceSpeed,
    ) -> IODeviceID {
        self.devices
            .add_device(addresses, device, interrupt_type, speed)
    }

    pub fn get_interrupt(&mut self, interrupt_type: InterruptType, clock: &Clock) -> bool {
        self.devices
            .get_by_interrupt_type(interrupt_type)
            .any(|device| device.get_interrupt(clock.get_cycles()))
    }

    pub fn set_interrupt(&mut self, device_id: IODeviceID, iterrupt: bool) {
        self.devices.get_by_id(device_id).set_interrupt(iterrupt);
    }

    pub fn read(&mut self, address: Word, clock: &mut Clock) -> u8 {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return 0xff;
        };

        access(|cycles| device.read(address, cycles), &config.speed, clock)
    }

    pub fn write(&mut self, address: Word, value: u8, clock: &mut Clock) {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return;
        };

        let needs_phase_2 = access(
            |cycles| device.write(address, value, cycles),
            &config.speed,
            clock,
        );

        if needs_phase_2 {
            self.phase_2_data = Some((address, value));
        }
    }

    pub fn phase_2(&mut self, clock: &Clock) {
        if let Some((address, value)) = self.phase_2_data {
            if let Some(device) = self.devices.get_by_address(address) {
                device.phase_2(address, value, clock.get_cycles());
            }

            self.phase_2_data = None;
        }
    }
}

pub fn access<F: FnOnce(u64) -> T, T>(access_fn: F, speed: &DeviceSpeed, clock: &mut Clock) -> T {
    match speed {
        DeviceSpeed::OneMhz => {
            clock.one_mhz_sync();

            let value = access_fn(clock.get_cycles());

            clock.inc();

            value
        }
        DeviceSpeed::TwoMhz => access_fn(clock.get_cycles()),
    }
}
