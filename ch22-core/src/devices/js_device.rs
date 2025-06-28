use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::interrupt_type::InterruptType;
use crate::word::Word;

use super::io_device::Ch22IODevice;

const JS_DEVICE_SLOW: u8 = 0b0000_0001;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0000_0010;
const JS_DEVICE_SYNC: u8 = 0b0000_0100;
const JS_DEVICE_NMI: u8 = 0b0001_0000;
const JS_DEVICE_IRQ: u8 = 0b0010_0000;

pub struct JsCh22Device {
    read: Box<dyn Fn(u16, u32) -> u64>,
    write: Box<dyn Fn(u16, u8, u32) -> u64>,
    handle_trigger: Box<dyn Fn(u32) -> u64>,
    wrap_trigger: Box<dyn Fn(u32)>,
    flags: u8,
    trigger: Option<u32>,
    phase_2_data: Option<(Word, u8)>,
    interrupt: bool,
    interrupt_type: Option<InterruptType>,
}

impl JsCh22Device {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        js_wrap_trigger: Function,
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

        let wrap_trigger = Box::new(move |wrap: u32| {
            js_wrap_trigger
                .call1(&JsValue::NULL, &wrap.into())
                .expect("js_wrap_trigger error");
        });

        let interrupt_type = match flags & (JS_DEVICE_IRQ | JS_DEVICE_NMI) {
            JS_DEVICE_IRQ => Some(InterruptType::IRQ),
            JS_DEVICE_NMI => Some(InterruptType::NMI),
            _ => None,
        };

        JsCh22Device {
            read,
            write,
            handle_trigger,
            wrap_trigger,
            flags,
            trigger: None,
            phase_2_data: None,
            interrupt: false,
            interrupt_type,
        }
    }
}

impl Ch22IODevice for JsCh22Device {
    fn read(&mut self, address: Word, cycles: u32) -> u8 {
        let value = self.set_js_device_params((self.read)(address.into(), cycles));

        (value & 0xff) as u8
    }

    fn write(&mut self, address: Word, value: u8, cycles: u32) -> bool {
        if self.flags & JS_DEVICE_PHASE_2_WRITE == 0 {
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
        if self.flags & JS_DEVICE_SYNC != 0 {
            self.sync_internal(cycles);
        }
    }

    fn is_slow(&self) -> bool {
        self.flags & JS_DEVICE_SLOW != 0
    }

    fn get_interrupt(&mut self, interrupt_type: InterruptType, cycles: u32) -> bool {
        if self.interrupt_type.is_none() || self.interrupt_type != Some(interrupt_type) {
            return false;
        }

        self.sync_internal(cycles);

        return self.interrupt;
    }

    fn set_interrupt(&mut self, interrupt: bool) {
        if self.interrupt_type.is_none() {
            return;
        }

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
