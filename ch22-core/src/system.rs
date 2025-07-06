use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::cpu::*;
use crate::cycle_manager::*;
use crate::device_map::DeviceMap;
use crate::devices::constant_device::*;
use crate::devices::io_space::*;
use crate::devices::js_device::*;
use crate::devices::js_sync_device::*;
use crate::interrupt_type::InterruptType;
use crate::utils;

#[wasm_bindgen]
pub struct Ch22System {
    cpu: Ch22Cpu,
    cycle_manager: CycleManager,
}

#[wasm_bindgen]
impl Ch22System {
    pub fn new() -> Ch22System {
        utils::set_panic_hook();

        let device_map = DeviceMap::new();

        let cycle_manager = CycleManager::new(device_map);

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
        slow: bool,
        panic_on_write: bool,
    ) -> DeviceID {
        self.cycle_manager.device_map.io_space.add_device(
            addresses,
            Box::new(Ch22ConstantDevice {
                read_value,
                panic_on_write,
            }),
            None,
            slow,
        )
    }

    pub fn add_device_js(
        &mut self,
        addresses: &[u16],
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        flags: u8,
    ) -> DeviceID {
        let interrupt_type = match flags & (JS_DEVICE_IRQ | JS_DEVICE_NMI) {
            JS_DEVICE_IRQ => Some(InterruptType::IRQ),
            JS_DEVICE_NMI => Some(InterruptType::NMI),
            _ => None,
        };

        self.cycle_manager.device_map.io_space.add_device(
            addresses,
            Box::new(JsCh22Device::new(
                js_read,
                js_write,
                js_handle_trigger,
                flags & JS_DEVICE_PHASE_2_WRITE != 0,
            )),
            interrupt_type,
            flags & JS_DEVICE_SLOW != 0,
        )
    }

    pub fn add_js_sync_device(&mut self, js_handle_trigger: Function) -> DeviceID {
        self.cycle_manager
            .add_device(Box::new(JsCh22SyncDevice::new(js_handle_trigger)))
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

    pub fn run(&mut self, run_until: u64) -> u64 {
        while self.cycle_manager.cycles < run_until {
            self.cpu.handle_next_instruction(&mut self.cycle_manager);
        }

        self.cycle_manager.cycles
    }

    pub fn set_device_interrupt(&mut self, device_id: DeviceID, interrupt: bool) {
        self.cycle_manager
            .device_map
            .io_space
            .set_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: DeviceID, trigger: Option<u64>) {
        self.cycle_manager.set_device_trigger(device_id, trigger);
    }
}

const JS_DEVICE_SLOW: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
