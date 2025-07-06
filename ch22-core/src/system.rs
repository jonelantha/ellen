use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::address_spaces::address_map::*;
use crate::address_spaces::io_space::*;
use crate::cpu::*;
use crate::cycle_manager::*;
use crate::interrupt_type::InterruptType;
use crate::io_devices::js_io_device::JsIODevice;
use crate::io_devices::static_device::StaticDevice;
use crate::timer_devices::js_timer_device::*;
use crate::timer_devices::timer_device_list::TimerDeviceID;
use crate::utils;

#[wasm_bindgen]
pub struct System {
    cpu: Cpu,
    cycle_manager: CycleManager,
}

#[wasm_bindgen]
impl System {
    pub fn new() -> System {
        utils::set_panic_hook();

        let device_map = AddressMap::new();

        let cycle_manager = CycleManager::new(device_map);

        System {
            cpu: Cpu::new(),
            cycle_manager,
        }
    }

    pub fn load_os_rom(&mut self, data: &[u8]) {
        self.cycle_manager.address_map.os_rom.load(data);
    }

    pub fn load_paged_rom(&mut self, bank: u8, data: &[u8]) {
        self.cycle_manager.address_map.paged_rom.load(bank, data);
    }

    pub fn add_static_device(
        &mut self,
        addresses: &[u16],
        read_value: u8,
        slow: bool,
        panic_on_write: bool,
    ) -> IODeviceID {
        self.cycle_manager.address_map.io_space.add_device(
            addresses,
            Box::new(StaticDevice {
                read_value,
                panic_on_write,
            }),
            None,
            slow,
        )
    }

    pub fn add_js_io_device(
        &mut self,
        addresses: &[u16],
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        flags: u8,
    ) -> IODeviceID {
        let interrupt_type = match flags & (JS_DEVICE_IRQ | JS_DEVICE_NMI) {
            JS_DEVICE_IRQ => Some(InterruptType::IRQ),
            JS_DEVICE_NMI => Some(InterruptType::NMI),
            _ => None,
        };

        self.cycle_manager.address_map.io_space.add_device(
            addresses,
            Box::new(JsIODevice::new(
                js_read,
                js_write,
                js_handle_trigger,
                flags & JS_DEVICE_PHASE_2_WRITE != 0,
            )),
            interrupt_type,
            flags & JS_DEVICE_SLOW != 0,
        )
    }

    pub fn add_js_timer_device(&mut self, js_handle_trigger: Function) -> TimerDeviceID {
        self.cycle_manager
            .timer_devices
            .add_device(Box::new(JsTimerDevice::new(js_handle_trigger)))
    }

    pub fn ram_start(&self) -> *const u8 {
        self.cycle_manager.address_map.ram.ram_start()
    }

    pub fn ram_size(&self) -> usize {
        self.cycle_manager.address_map.ram.ram_size()
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

    pub fn set_device_interrupt(&mut self, device_id: IODeviceID, interrupt: bool) {
        self.cycle_manager
            .address_map
            .io_space
            .set_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.cycle_manager
            .timer_devices
            .set_device_trigger(device_id, trigger);
    }
}

const JS_DEVICE_SLOW: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
