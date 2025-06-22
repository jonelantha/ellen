use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::word::Word;

use super::io_device::Ch22IODevice;

pub struct JsCh22DeviceExt {
    read: Box<dyn Fn(u16, u32) -> u64>,
    write: Box<dyn Fn(u16, u8, u32) -> u64>,
    handle_trigger: Box<dyn Fn(u32) -> u64>,
    phase_2: Option<Box<dyn Fn(u32) -> u64>>,
    is_slow: bool,
    trigger: Option<u32>,
}

impl JsCh22DeviceExt {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        js_phase_2: Option<Function>,
        is_slow: bool,
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

        let phase_2 = js_phase_2.map(|js_phase_2| {
            Box::new(move |cycles: u32| {
                js_phase_2
                    .call1(&JsValue::NULL, &cycles.into())
                    .expect("js_phase_2 error")
                    .try_into()
                    .expect("js_handle_trigger error")
            }) as Box<dyn Fn(u32) -> u64>
        });

        let handle_trigger = Box::new(move |cycles: u32| {
            js_handle_trigger
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_handle_trigger error")
                .try_into()
                .expect("js_handle_trigger error")
        });

        JsCh22DeviceExt {
            read,
            write,
            handle_trigger,
            phase_2,
            is_slow,
            trigger: None,
        }
    }
}

impl Ch22IODevice for JsCh22DeviceExt {
    fn read(&mut self, address: Word, cycles: u32, interrupt: &mut u8) -> u8 {
        let value = self.set_js_device_params(interrupt, (self.read)(address.into(), cycles));

        (value & 0xff) as u8
    }

    fn write(&mut self, address: Word, value: u8, cycles: u32, interrupt: &mut u8) -> bool {
        self.set_js_device_params(interrupt, (self.write)(address.into(), value, cycles));

        self.phase_2.is_some()
    }

    fn phase_2(&mut self, _address: Word, cycles: u32, interrupt: &mut u8) {
        if let Some(phase_2) = &self.phase_2 {
            self.set_js_device_params(interrupt, (phase_2)(cycles));
        }
    }

    fn is_slow(&self) -> bool {
        self.is_slow
    }

    fn sync(&mut self, cycles: u32, interrupt: &mut u8) {
        if let Some(trigger) = self.trigger {
            if trigger <= cycles {
                self.set_js_device_params(interrupt, (self.handle_trigger)(cycles));
            }
        }
    }
}

impl JsCh22DeviceExt {
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
