use std::mem::size_of;

use js_sys::Function;
use wasm_bindgen::prelude::*;

use super::core::{Core, ROMS_LEN};
use crate::cpu::InterruptType;
use crate::devices::{DeviceID, DeviceSpeed, JsDevice, StaticDevice, new_sys_via_stub};
use crate::sound_register_writes::SoundRegisterWrites;
use crate::utils;
use crate::video::Field;

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
        self.core.video.get_field_start()
    }

    pub fn video_field_size(&self) -> usize {
        size_of::<Field>()
    }

    pub fn sound_register_writes_start(&mut self) -> *const SoundRegisterWrites {
        self.core.get_sound_register_writes_start()
    }

    pub fn sound_register_writes_size(&self) -> usize {
        size_of::<SoundRegisterWrites>()
    }

    pub fn load_rom(&mut self, bank: usize, data: &[u8]) {
        if bank >= ROMS_LEN {
            panic!("Invalid ROM bank: {bank}");
        }

        self.core.roms[bank].load(data);
    }

    pub fn add_static_device(
        &mut self,
        addresses: &[u16],
        read_value: u8,
        one_mhz: bool,
        panic_on_write: bool,
    ) -> DeviceID {
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

    pub fn add_js_device(
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

        let speed = match flags & JS_DEVICE_ONE_MHZ {
            JS_DEVICE_ONE_MHZ => DeviceSpeed::OneMhz,
            _ => DeviceSpeed::TwoMhz,
        };

        self.core.io_space.add_device(
            addresses,
            Box::new(JsDevice::new(
                js_read,
                js_write,
                js_handle_trigger,
                flags & JS_DEVICE_PHASE_2_WRITE != 0,
            )),
            interrupt_type,
            speed,
        )
    }

    pub fn add_sys_via_stub(
        &mut self,
        addresses: &[u16],
        js_read: Function,
        js_write: Function,
        js_on_vsync_change: Function,
        js_handle_trigger: Function,
    ) -> DeviceID {
        let sys_via_bus = self.core.get_sys_via_bus();

        self.core.io_space.add_device(
            addresses,
            Box::new(new_sys_via_stub(
                js_read,
                js_write,
                js_on_vsync_change,
                js_handle_trigger,
                sys_via_bus,
            )),
            Some(InterruptType::IRQ),
            DeviceSpeed::OneMhz,
        )
    }

    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub fn run_one_field(&mut self) -> u64 {
        self.core.run_one_field()
    }

    pub fn set_device_interrupt(&mut self, device_id: DeviceID, interrupt: bool) {
        self.core.io_space.set_interrupt(device_id, interrupt);
    }
}

const JS_DEVICE_ONE_MHZ: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
