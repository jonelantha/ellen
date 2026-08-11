use std::collections::HashMap;

use crate::cpu::InterruptType;
use crate::devices::Device;
use crate::word::Word;

pub type DeviceID = usize;

pub enum DeviceSpeed {
    OneMhz,
    TwoMhz,
}

#[derive(Default)]
pub struct DeviceList {
    device_list: Vec<Box<dyn Device>>,
    address_to_device_id: HashMap<Word, DeviceID>,
    config_list: Vec<DeviceConfig>,
}

impl DeviceList {
    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn Device>,
        interrupt_type: Option<InterruptType>,
        speed: DeviceSpeed,
    ) -> DeviceID {
        self.device_list.push(device);

        self.config_list.push(DeviceConfig {
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

    pub fn get_by_id(&mut self, device_id: DeviceID) -> &mut dyn Device {
        self.device_list[device_id].as_mut()
    }

    pub fn get_with_config_by_id(
        &mut self,
        device_id: DeviceID,
    ) -> (&mut dyn Device, &DeviceConfig) {
        let device = self.device_list[device_id].as_mut();
        let config = &self.config_list[device_id];

        (device, config)
    }

    pub fn get_by_address(&mut self, address: Word) -> Option<&mut dyn Device> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_by_id(*device_id))
    }

    pub fn get_with_config_by_address(
        &mut self,
        address: Word,
    ) -> Option<(&mut dyn Device, &DeviceConfig)> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_with_config_by_id(*device_id))
    }

    pub fn get_by_interrupt_type(
        &mut self,
        interrupt_type: InterruptType,
    ) -> impl Iterator<Item = &mut Box<dyn Device>> {
        let config_list = &self.config_list;

        self.device_list
            .iter_mut()
            .enumerate()
            .filter(move |(device_id, _)| {
                config_list[*device_id].interrupt_type == Some(interrupt_type)
            })
            .map(|(_, device)| device)
    }

    pub fn for_each<F: FnMut(&mut Box<dyn Device>)>(&mut self, mut callback: F) {
        for device in self.device_list.iter_mut() {
            callback(device);
        }
    }
}

pub struct DeviceConfig {
    pub interrupt_type: Option<InterruptType>,
    pub speed: DeviceSpeed,
}
