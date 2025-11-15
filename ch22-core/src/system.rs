mod address_map;
mod clock;
mod cpu_bus;
mod system_components;
mod system_runner;

use std::mem::size_of;

use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::cpu::InterruptType;
use crate::devices::{
    DeviceSpeed, IODeviceID, JsIODevice, JsTimerDevice, RomSelect, StaticDevice, TimerDeviceID,
};
use crate::utils;
use crate::video::{CRTCRangeType, Field};

use system_components::SystemComponents;

pub use clock::Clock;

#[wasm_bindgen]
#[derive(Default)]
pub struct System {
    system_components: SystemComponents,
}

#[wasm_bindgen]
impl System {
    pub fn new() -> System {
        utils::set_panic_hook();

        let mut system = Self::default();

        let paged_rom_select = system.system_components.paged_rom().get_active_rom();

        system.system_components.io_space().add_device(
            &[0xfe30, 0xfe31, 0xfe32, 0xfe33],
            Box::new(RomSelect::new(paged_rom_select)),
            None,
            DeviceSpeed::TwoMhz,
        );

        system
    }

    pub fn video_field_start(&mut self) -> *const Field {
        self.system_components.video_field() as *const Field
    }

    pub fn video_field_size(&self) -> usize {
        size_of::<Field>()
    }

    pub fn snapshot_char_data(
        &mut self,
        row_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        is_teletext: bool,
    ) {
        self.system_components.snapshot_char_data(
            row_index,
            crtc_address,
            crtc_length,
            match is_teletext {
                true => CRTCRangeType::Teletext,
                false => CRTCRangeType::HiRes,
            },
        );
    }

    pub fn load_os_rom(&mut self, data: &[u8]) {
        self.system_components.os_rom().load(data);
    }

    pub fn load_paged_rom(&mut self, bank: u8, data: &[u8]) {
        self.system_components.paged_rom().load(bank, data);
    }

    pub fn add_static_device(
        &mut self,
        addresses: &[u16],
        read_value: u8,
        one_mhz: bool,
        panic_on_write: bool,
    ) -> IODeviceID {
        let speed = match one_mhz {
            true => DeviceSpeed::OneMhz,
            false => DeviceSpeed::TwoMhz,
        };

        self.system_components.io_space().add_device(
            addresses,
            Box::new(StaticDevice {
                read_value,
                panic_on_write,
            }),
            None,
            speed,
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

        let speed = match flags & JS_DEVICE_ONE_MHZ {
            JS_DEVICE_ONE_MHZ => DeviceSpeed::OneMhz,
            _ => DeviceSpeed::TwoMhz,
        };

        let ic32_latch = self.system_components.clone_ic32_latch();
        self.system_components.io_space().add_device(
            addresses,
            Box::new(JsIODevice::new(
                js_read,
                js_write,
                js_handle_trigger,
                flags & JS_DEVICE_PHASE_2_WRITE != 0,
                ic32_latch,
            )),
            interrupt_type,
            speed,
        )
    }

    pub fn add_js_timer_device(&mut self, js_handle_trigger: Function) -> TimerDeviceID {
        self.system_components
            .timer_devices()
            .add_device(Box::new(JsTimerDevice::new(js_handle_trigger)))
    }

    pub fn reset(&mut self) {
        self.system_components.reset();
    }

    pub fn run(&mut self, until: u64) -> u64 {
        self.system_components.run(until)
    }

    pub fn set_device_interrupt(&mut self, device_id: IODeviceID, interrupt: bool) {
        self.system_components
            .io_space()
            .set_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.system_components
            .timer_devices()
            .set_device_trigger(device_id, trigger);
    }
}

const JS_DEVICE_ONE_MHZ: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
