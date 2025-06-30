use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::word::Word;

use super::io_device::Ch22IODevice;

pub struct JsCh22Device {
    read: Box<dyn Fn(u16, u32) -> u64>,
    write: Box<dyn Fn(u16, u8, u32) -> u64>,
    handle_trigger: Box<dyn Fn(u32) -> u64>,
    wrap_trigger: Box<dyn Fn(u32)>,
    trigger: Option<u32>,
    phase_2_data: Option<(Word, u8)>,
    interrupt: bool,
    requires_sync: bool,
    phase_2_write: bool,
}

impl JsCh22Device {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        js_wrap_trigger: Function,
        requires_sync: bool,
        phase_2_write: bool,
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

        let wrap_trigger = Box::new(move |wrap: u32| {
            js_wrap_trigger
                .call1(&JsValue::NULL, &wrap.into())
                .expect("js_wrap_trigger error");
        });

        JsCh22Device {
            read,
            write,
            handle_trigger,
            wrap_trigger,
            trigger: None,
            phase_2_data: None,
            interrupt: false,
            requires_sync,
            phase_2_write,
        }
    }
}

impl Ch22IODevice for JsCh22Device {
    fn read(&mut self, address: Word, cycles: u32) -> u8 {
        let value = self.set_js_device_params((self.read)(address.into(), cycles));

        (value & 0xff) as u8
    }

    fn write(&mut self, address: Word, value: u8, cycles: u32) -> bool {
        if !self.phase_2_write {
            self.set_js_device_params((self.write)(address.into(), value, cycles));
            false
        } else {
            self.phase_2_data = Some((address, value));

            true
        }
    }

    fn phase_2(&mut self, cycles: u32) {
        let (address, value) = self.phase_2_data.unwrap();

        self.set_js_device_params((self.write)(address.into(), value, cycles));
    }

    fn sync(&mut self, cycles: u32) {
        if self.requires_sync {
            self.sync_internal(cycles);
        }
    }

    fn get_interrupt(&mut self, cycles: u32) -> bool {
        self.sync_internal(cycles);

        self.interrupt
    }

    fn set_interrupt(&mut self, interrupt: bool) {
        self.interrupt = interrupt;
    }

    fn set_trigger(&mut self, trigger: Option<u32>) {
        self.trigger = trigger;
    }

    fn wrap_trigger(&mut self, wrap: u32) {
        if let Some(trigger) = self.trigger {
            self.trigger = Some(trigger - wrap);
        }
        (self.wrap_trigger)(wrap);
    }
}

impl JsCh22Device {
    fn sync_internal(&mut self, cycles: u32) {
        if let Some(trigger) = self.trigger {
            if trigger <= cycles {
                self.set_js_device_params((self.handle_trigger)(cycles));
            }
        }
    }

    // trig trig trig trig null null flags value

    fn set_js_device_params(&mut self, params_and_value: u64) -> u8 {
        let flags = (params_and_value >> 8) & 0xff;
        let value = ((params_and_value) & 0xff) as u8;

        self.interrupt = flags & 0x02 != 0;

        self.trigger = if flags & 0x01 != 0 {
            Some((params_and_value >> 32) as u32)
        } else {
            None
        };

        value
    }
}
