use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::address_map::io_space::DeviceSpeed;
use crate::address_map::io_space::io_device_list::IODeviceID;
use crate::devices::js_io_device::JsIODevice;
use crate::devices::js_timer_device::*;
use crate::devices::static_device::StaticDevice;
use crate::devices::timer_device_list::TimerDeviceID;
use crate::interrupt_type::InterruptType;
use crate::system_state::SystemState;
use crate::utils;
use crate::video::field_data::Field;
use crate::video::video_memory_access::CRTCRangeType;
use std::mem::size_of;

#[wasm_bindgen]
#[derive(Default)]
pub struct System {
    system_state: SystemState,
}

#[wasm_bindgen]
impl System {
    pub fn new() -> System {
        utils::set_panic_hook();

        System::default()
    }

    pub fn video_field_start(&self) -> *const Field {
        self.system_state.video_field_start()
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
        self.system_state.snapshot_char_data(
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
        self.system_state.load_os_rom(data);
    }

    pub fn load_paged_rom(&mut self, bank: u8, data: &[u8]) {
        self.system_state.load_paged_rom(bank, data);
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

        self.system_state.add_io_device(
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

        self.system_state.add_io_device(
            addresses,
            Box::new(JsIODevice::new(
                js_read,
                js_write,
                js_handle_trigger,
                flags & JS_DEVICE_PHASE_2_WRITE != 0,
                self.system_state.clone_ic32_latch(),
            )),
            interrupt_type,
            speed,
        )
    }

    pub fn add_js_timer_device(&mut self, js_handle_trigger: Function) -> TimerDeviceID {
        self.system_state
            .add_timer_device(Box::new(JsTimerDevice::new(js_handle_trigger)))
    }

    pub fn reset(&mut self) {
        self.system_state.reset();
    }

    pub fn run(&mut self, until: u64) -> u64 {
        self.system_state.run(until)
    }

    pub fn set_device_interrupt(&mut self, device_id: IODeviceID, interrupt: bool) {
        self.system_state.set_device_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.system_state.set_device_trigger(device_id, trigger);
    }
}

const JS_DEVICE_ONE_MHZ: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
