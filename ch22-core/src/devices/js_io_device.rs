use js_sys::Function;
use wasm_bindgen::JsValue;

use super::io_device::*;
use crate::word::Word;

pub struct JsIODevice {
    read: Box<dyn Fn(u16, u64) -> u64>,
    write: Box<dyn Fn(u16, u8, u64) -> u64>,
    handle_trigger: Box<dyn Fn(u64) -> u64>,
    trigger: Option<u64>,
    interrupt: bool,
    phase_2_write: bool,
}

impl JsIODevice {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        phase_2_write: bool,
    ) -> Self {
        let read = Box::new(move |address: u16, cycles: u64| {
            js_read
                .call2(&JsValue::NULL, &address.into(), &cycles.into())
                .expect("js_read error")
                .try_into()
                .expect("js_read error")
        });

        let write = Box::new(move |address: u16, value: u8, cycles: u64| {
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

        let handle_trigger = Box::new(move |cycles: u64| {
            js_handle_trigger
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_handle_trigger error")
                .try_into()
                .expect("js_handle_trigger error")
        });

        JsIODevice {
            read,
            write,
            handle_trigger,
            trigger: None,
            interrupt: false,
            phase_2_write,
        }
    }
}

impl IODevice for JsIODevice {
    fn read(&mut self, address: Word, cycles: u64) -> u8 {
        self.set_js_device_params((self.read)(address.into(), cycles))
    }

    fn write(&mut self, address: Word, value: u8, cycles: u64) -> bool {
        if !self.phase_2_write {
            self.set_js_device_params((self.write)(address.into(), value, cycles));

            false
        } else {
            true
        }
    }

    fn phase_2(&mut self, address: Word, value: u8, cycles: u64) {
        if self.phase_2_write {
            self.set_js_device_params((self.write)(address.into(), value, cycles));
        }
    }

    fn get_interrupt(&mut self, cycles: u64) -> bool {
        self.sync(cycles);

        self.interrupt
    }

    fn set_interrupt(&mut self, interrupt: bool) {
        self.interrupt = interrupt;
    }
}

impl JsIODevice {
    fn sync(&mut self, cycles: u64) {
        if let Some(trigger) = self.trigger
            && trigger <= cycles
        {
            self.set_js_device_params((self.handle_trigger)(cycles));
        }
    }

    // trig trig trig trig trig trig flags value

    fn set_js_device_params(&mut self, params_and_value: u64) -> u8 {
        let [_, _, _, _, _, _, flags, value] = params_and_value.to_be_bytes();

        self.interrupt = flags & JS_IO_FLAG_INTERRUPT != 0;

        self.trigger = if flags & JS_IO_FLAG_HAS_TRIGGER != 0 {
            Some(params_and_value >> 16)
        } else {
            None
        };

        value
    }
}

const JS_IO_FLAG_HAS_TRIGGER: u8 = 0x01;
const JS_IO_FLAG_INTERRUPT: u8 = 0x02;
