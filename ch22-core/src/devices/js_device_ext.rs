use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::word::Word;

use super::io_device::Ch22IODevice;

pub struct JsCh22DeviceExt {
    read: Box<dyn Fn(u16, u32) -> u64>,
    write: Box<dyn Fn(u16, u8, u32) -> u64>,
    handle_trigger: Box<dyn Fn(u32) -> u64>,
    phase_2: Option<Box<dyn Fn(u32)>>,
    is_slow: bool,
    interrupt: u16,
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
                    .expect("js_phase_2 error");
            }) as Box<dyn Fn(u32)>
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
            interrupt: 0,
            trigger: None,
        }
    }
}

impl Ch22IODevice for JsCh22DeviceExt {
    fn read(&mut self, address: Word, cycles: u32) -> u8 {
        let value = self.set_js_device_params((self.read)(address.into(), cycles));

        (value & 0xff) as u8
    }

    fn write(&mut self, address: Word, value: u8, cycles: u32) -> bool {
        self.set_js_device_params((self.write)(address.into(), value, cycles));

        self.phase_2.is_some()
    }

    fn phase_2(&mut self, _address: Word, cycles: u32) {
        if let Some(phase_2) = &self.phase_2 {
            (phase_2)(cycles);
        }
    }

    fn is_slow(&self) -> bool {
        self.is_slow
    }

    fn get_interrupt(&mut self, cycles: u32) -> u16 {
        if let Some(trigger) = self.trigger {
            if trigger <= cycles {
                self.set_js_device_params((self.handle_trigger)(cycles));
            }
        }

        self.interrupt
    }
}

impl JsCh22DeviceExt {
    // trig trig trig trig irq nmi flags value

    fn set_js_device_params(&mut self, params_and_value: u64) -> u8 {
        self.interrupt = ((params_and_value >> 16) & 0xffff) as u16;

        let flags = (params_and_value >> 8) & 0xff;

        self.trigger = if flags & 0x01 != 0 {
            Some((params_and_value >> 32) as u32)
        } else {
            None
        };

        (params_and_value & 0xff) as u8
    }
}
