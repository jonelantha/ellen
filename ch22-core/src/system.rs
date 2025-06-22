use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::cpu::*;
use crate::cycle_manager::*;
use crate::device_map::DeviceMap;
use crate::devices::constant_device::*;
use crate::devices::js_device::*;
use crate::devices::js_device_ext::*;
use crate::utils;

#[wasm_bindgen]
pub struct Ch22System {
    cpu: Ch22Cpu,
    cycle_manager: CycleManager,
}

#[wasm_bindgen]
impl Ch22System {
    pub fn new(js_wrap_counts: Function) -> Ch22System {
        utils::set_panic_hook();

        let wrap_counts = Box::new(move |wrap: u32| {
            js_wrap_counts
                .call1(&JsValue::NULL, &wrap.into())
                .expect("js_wrap_counts error");
        });

        let device_map = DeviceMap::new();

        let cycle_manager = CycleManager::new(device_map, wrap_counts);

        Ch22System {
            cpu: Ch22Cpu::new(),
            cycle_manager,
        }
    }

    pub fn load_os_rom(&mut self, data: &[u8]) {
        self.cycle_manager.device_map.os_rom.load(data);
    }

    pub fn load_paged_rom(&mut self, bank: u8, data: &[u8]) {
        self.cycle_manager.device_map.paged_rom.load(bank, data);
    }

    pub fn add_constant_device(
        &mut self,
        addresses: &[u16],
        read_value: u8,
        is_slow: bool,
        panic_on_write: bool,
    ) -> u8 {
        self.cycle_manager.device_map.io_space.add_device(
            addresses,
            Box::new(Ch22ConstantDevice {
                read_value,
                is_slow,
                panic_on_write,
            }),
        )
    }

    pub fn add_device_js(
        &mut self,
        addresses: &[u16],
        js_read: Function,
        js_write: Function,
        is_slow: bool,
        js_write_phase_2: Option<Function>,
    ) -> u8 {
        self.cycle_manager.device_map.io_space.add_device(
            addresses,
            Box::new(JsCh22Device::new(
                js_read,
                js_write,
                js_write_phase_2,
                is_slow,
            )),
        )
    }

    pub fn add_device_js_ext(
        &mut self,
        addresses: &[u16],
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        is_slow: bool,
        js_write_phase_2: Option<Function>,
    ) -> u8 {
        self.cycle_manager.device_map.io_space.add_device(
            addresses,
            Box::new(JsCh22DeviceExt::new(
                js_read,
                js_write,
                js_handle_trigger,
                js_write_phase_2,
                is_slow,
            )),
        )
    }

    pub fn ram_start(&self) -> *const u8 {
        self.cycle_manager.device_map.ram.ram_start()
    }

    pub fn ram_size(&self) -> usize {
        self.cycle_manager.device_map.ram.ram_size()
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.cycle_manager);
    }

    pub fn handle_next_instruction(&mut self) -> u32 {
        self.cpu.handle_next_instruction(&mut self.cycle_manager);

        self.cycle_manager.cycles
    }

    pub fn set_interrupt(&mut self, mask: u8, interrupts: u8) {
        self.cycle_manager
            .device_map
            .io_space
            .set_interrupt(mask, interrupts);
    }

    pub fn set_device_trigger(&mut self, device_id: u8, trigger: Option<u32>) {
        self.cycle_manager
            .device_map
            .io_space
            .set_device_trigger(device_id, trigger);
    }
}
