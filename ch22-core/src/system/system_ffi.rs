use std::mem::size_of;

use js_sys::Function;
use wasm_bindgen::prelude::*;

use super::{core::Core, core::OS_ROM};
use crate::cpu::InterruptType;
use crate::devices::{
    DeviceSpeed, IODeviceID, JsIODevice, JsTimerDevice, StaticDevice, TimerDeviceID,
};
use crate::utils;
use crate::video::{CRTCRangeType, Field};

#[wasm_bindgen(js_name = System)]
#[derive(Default)]
pub struct SystemFfi {
    core: Core,
}

#[wasm_bindgen(js_class = System)]
impl SystemFfi {
    pub fn new() -> SystemFfi {
        utils::set_panic_hook();

        let mut system_ffi = Self::default();

        system_ffi.core.setup();

        system_ffi
    }

    pub fn video_field_start(&mut self) -> *const Field {
        &self.core.video_field as *const Field
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
        self.core.snapshot_char_data(
            row_index,
            crtc_address,
            crtc_length,
            match is_teletext {
                true => CRTCRangeType::Teletext,
                false => CRTCRangeType::HiRes,
            },
        );
    }

    pub fn load_rom(&mut self, bank: u8, data: &[u8]) {
        self.core.roms[bank as usize].load(data);
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

        self.core.io_space.add_device(
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

        let ic32_latch = self.core.ic32_latch.clone();
        self.core.io_space.add_device(
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
        self.core
            .timer_devices
            .add_device(Box::new(JsTimerDevice::new(js_handle_trigger)))
    }

    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub fn run(&mut self, until: u64) -> u64 {
        self.core.run(until)
    }

    pub fn set_device_interrupt(&mut self, device_id: IODeviceID, interrupt: bool) {
        self.core.io_space.set_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.core
            .timer_devices
            .set_device_trigger(device_id, trigger);
    }
}

const JS_DEVICE_ONE_MHZ: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
