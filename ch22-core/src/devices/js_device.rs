use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::word::Word;

use super::io_device::Ch22IODevice;

const JS_DEVICE_SLOW: u8 = 0b00000001;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b00000010;

pub struct JsCh22Device {
    read: Box<dyn Fn(u16, u32) -> u64>,
    write: Box<dyn Fn(u16, u8, u32) -> u64>,
    handle_trigger: Box<dyn Fn(u32) -> u64>,
    flags: u8,
    trigger: Option<u32>,
    phase_2_data: Option<(Word, u8)>,
}

impl JsCh22Device {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        flags: u8,
    ) -> Self {
        let read = Box::new(move |address: u16, cycles: u32| {
            js_read
                .call2(&JsValue::NULL, &address.into(), &cycles.into())
                .expect("js_read error")
                .try_into()
                .expect("js_read error")
        });

        let write = Box::new(move |address: u16, value: u8, cycles: u32| {
            js_write
                .call3(
                    &JsValue::NULL,
                    &address.into(),
                    &value.into(),
                    &cycles.into(),
                )
                .expect("js_write error")
                .try_into()
                .expect("js_write error")
        });

        let handle_trigger = Box::new(move |cycles: u32| {
            js_handle_trigger
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_handle_trigger error")
                .try_into()
                .expect("js_handle_trigger error")
        });

        JsCh22Device {
            read,
            write,
            handle_trigger,
            flags,
            trigger: None,
            phase_2_data: None,
        }
    }
}

impl Ch22IODevice for JsCh22Device {
    fn read(&mut self, address: Word, cycles: u32, interrupt: &mut u8) -> u8 {
        let value = self.set_js_device_params(interrupt, (self.read)(address.into(), cycles));

        (value & 0xff) as u8
    }

    fn write(&mut self, address: Word, value: u8, cycles: u32, interrupt: &mut u8) -> bool {
        if self.flags & JS_DEVICE_PHASE_2_WRITE == 0 {
            self.set_js_device_params(interrupt, (self.write)(address.into(), value, cycles));
            false
        } else {
            self.phase_2_data = Some((address, value));

            true
        }
    }

    fn phase_2(&mut self, cycles: u32, interrupt: &mut u8) {
        let (address, value) = self.phase_2_data.unwrap();

        self.set_js_device_params(interrupt, (self.write)(address.into(), value, cycles));
    }

    fn is_slow(&self) -> bool {
        self.flags & JS_DEVICE_SLOW != 0
    }

    fn sync(&mut self, cycles: u32, interrupt: &mut u8) {
        if let Some(trigger) = self.trigger {
            if trigger <= cycles {
                self.set_js_device_params(interrupt, (self.handle_trigger)(cycles));
            }
        }
    }

    fn set_trigger(&mut self, trigger: Option<u32>) {
        self.trigger = trigger;
    }
}

impl JsCh22Device {
    // trig trig trig trig intmask intval flags value

    fn set_js_device_params(&mut self, interrupt: &mut u8, params_and_value: u64) -> u8 {
        let interrupt_mask = ((params_and_value >> 24) & 0xff) as u8;
        let interrupt_flags = ((params_and_value >> 16) & 0xff) as u8;
        let flags = (params_and_value >> 8) & 0xff;
        let value = ((params_and_value) & 0xff) as u8;

        *interrupt = (*interrupt & !interrupt_mask) | (interrupt_flags & interrupt_mask);

        self.trigger = if flags & 0x01 != 0 {
            Some((params_and_value >> 32) as u32)
        } else {
            None
        };

        value
    }
}
