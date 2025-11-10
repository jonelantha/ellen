use std::collections::HashMap;

use crate::address_map::io_space::DeviceSpeed;
use crate::devices::IODevice;
use crate::interrupt_type::InterruptType;
use crate::word::Word;

pub type IODeviceID = usize;

#[derive(Default)]
pub struct IODeviceList {
    device_list: Vec<Box<dyn IODevice>>,
    address_to_device_id: HashMap<Word, IODeviceID>,
    config_list: Vec<IODeviceConfig>,
}

impl IODeviceList {
    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        speed: DeviceSpeed,
    ) -> IODeviceID {
        self.device_list.push(device);

        self.config_list.push(IODeviceConfig {
            interrupt_type,
            speed,
        });

        // assumes devices will not be removed
        let device_id = self.device_list.len() - 1;

        for address in addresses {
            self.address_to_device_id
                .insert((*address).into(), device_id);
        }

        device_id
    }

    pub fn get_by_id(&mut self, device_id: IODeviceID) -> &mut dyn IODevice {
        self.device_list[device_id].as_mut()
    }

    pub fn get_with_config_by_id(
        &mut self,
        device_id: IODeviceID,
    ) -> (&mut dyn IODevice, &IODeviceConfig) {
        let device = self.device_list[device_id].as_mut();
        let config = &self.config_list[device_id];

        (device, config)
    }

    pub fn get_by_address(&mut self, address: Word) -> Option<&mut dyn IODevice> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_by_id(*device_id))
    }

    pub fn get_with_config_by_address(
        &mut self,
        address: Word,
    ) -> Option<(&mut dyn IODevice, &IODeviceConfig)> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_with_config_by_id(*device_id))
    }

    pub fn get_by_interrupt_type(
        &mut self,
        interrupt_type: InterruptType,
    ) -> impl Iterator<Item = &mut Box<dyn IODevice>> {
        let config_list = &self.config_list;

        self.device_list
            .iter_mut()
            .enumerate()
            .filter(move |(device_id, _)| {
                config_list[*device_id].interrupt_type == Some(interrupt_type)
            })
            .map(|(_, device)| device)
    }
}

pub struct IODeviceConfig {
    pub interrupt_type: Option<InterruptType>,
    pub speed: DeviceSpeed,
}
