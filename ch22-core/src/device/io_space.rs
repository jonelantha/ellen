use std::{collections::HashMap, ops::RangeInclusive};

use js_sys::Function;
use wasm_bindgen::prelude::*;

use super::device::Ch22Device;
use super::js_device::*;
use super::paged_rom::*;

#[wasm_bindgen]
pub struct Ch22IOSpace {
    device_list: DeviceList2,
}

#[wasm_bindgen]
impl Ch22IOSpace {
    pub fn new() -> Self {
        Ch22IOSpace {
            device_list: DeviceList2::new(),
        }
    }

    pub fn add_rom_select(
        &mut self,
        start_address: u16,
        end_address: u16,
        rom_select: Ch22RomSelect,
    ) {
        self.device_list
            .add_device(start_address..=end_address, Box::new(rom_select))
    }

    pub fn add_device_js(
        &mut self,
        start_address: u16,
        end_address: u16,
        js_read: Function,
        js_write: Function,
        is_slow: bool,
        js_write_phase_2: Option<Function>,
    ) {
        self.device_list.add_device(
            start_address..=end_address,
            Box::new(JsCh22Device::new(
                js_read,
                js_write,
                js_write_phase_2,
                is_slow,
            )),
        )
    }
}

impl Ch22Device for Ch22IOSpace {
    fn read(&mut self, address: u16, cycles: u32) -> u8 {
        if let Some(device) = self.device_list.get_device_mut(address) {
            device.read(address, cycles)
        } else {
            0xff
        }
    }

    fn is_slow(&self, address: u16) -> bool {
        if let Some(device) = self.device_list.get_device(address) {
            device.is_slow(address)
        } else {
            false
        }
    }

    fn write(&mut self, address: u16, value: u8, cycles: u32) -> bool {
        if let Some(device) = self.device_list.get_device_mut(address) {
            device.write(address, value, cycles)
        } else {
            false
        }
    }

    fn phase_2(&mut self, address: u16, cycles: u32) {
        if let Some(device) = self.device_list.get_device_mut(address) {
            device.phase_2(address, cycles);
        }
    }
}

pub struct DeviceList2 {
    next_device_id: u8,
    devices: HashMap<u8, Box<dyn Ch22Device>>,
    address_to_device_id: HashMap<u16, u8>,
}

impl DeviceList2 {
    pub fn new() -> Self {
        DeviceList2 {
            next_device_id: 0,
            devices: HashMap::new(),
            address_to_device_id: HashMap::new(),
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

    fn get_device_mut(&mut self, address: u16) -> Option<&mut dyn Ch22Device> {
        if let Some(device_id) = self.address_to_device_id.get(&address) {
            Some(self.devices.get_mut(device_id).unwrap().as_mut())
        } else {
            None
        }
    }

    fn get_device(&self, address: u16) -> Option<&dyn Ch22Device> {
        if let Some(device_id) = self.address_to_device_id.get(&address) {
            Some(self.devices.get(device_id).unwrap().as_ref())
        } else {
            None
        }
    }
}
