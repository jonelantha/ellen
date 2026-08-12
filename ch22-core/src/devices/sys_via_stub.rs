use std::{cell::Cell, rc::Rc};

use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::word::Word;

use super::device::Device;

pub struct SysViaStub {
    read: Box<dyn Fn(u16, u64) -> u64>,
    write: Box<dyn Fn(u16, u8, u64) -> u64>,
    on_vsync_change: Box<dyn Fn(bool) -> u64>,
    handle_trigger: Box<dyn Fn(u64) -> u64>,
    trigger: Option<u64>,
    interrupt: bool,
    ic32_latch: Rc<Cell<u8>>,
}

impl SysViaStub {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_on_vsync_change: Function,
        js_handle_trigger: Function,
        ic32_latch: Rc<Cell<u8>>,
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

        let on_vsync_change = Box::new(move |vsync: bool| {
            js_on_vsync_change
                .call1(&JsValue::NULL, &vsync.into())
                .expect("js_on_vsync_change error")
                .try_into()
                .expect("js_on_vsync_change error")
        });

        let handle_trigger = Box::new(move |cycles: u64| {
            js_handle_trigger
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_handle_trigger error")
                .try_into()
                .expect("js_handle_trigger error")
        });

        SysViaStub {
            read,
            write,
            on_vsync_change,
            handle_trigger,
            trigger: None,
            interrupt: false,
            ic32_latch,
        }
    }
}

impl Device for SysViaStub {
    fn read(&mut self, address: Word, cycles: u64) -> u8 {
        self.set_sys_via_stub_params((self.read)(address.into(), cycles))
            .unwrap()
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: u64) -> bool {
        true
    }

    fn phase_2(&mut self, address: Word, value: u8, cycles: u64) {
        self.set_sys_via_stub_params((self.write)(address.into(), value, cycles));
    }

    fn get_interrupt(&mut self, cycles: u64) -> bool {
        self.sync(cycles);

        self.interrupt
    }

    fn set_interrupt(&mut self, interrupt: bool) {
        self.interrupt = interrupt;
    }

    fn on_vsync_change(&mut self, vsync: bool) {
        self.set_sys_via_stub_params((self.on_vsync_change)(vsync));
    }
}

impl SysViaStub {
    fn sync(&mut self, cycles: u64) {
        if let Some(trigger) = self.trigger
            && trigger <= cycles
        {
            self.set_sys_via_stub_params((self.handle_trigger)(cycles));
        }
    }

    // Encoding format: [trig trig trig trig trig trig flags (value or ic32)]
    // The last byte contains either a value or ic32 data, depending on the JS_DEVICE_FLAG_VALUE_IS_IC32 flag.
    fn set_sys_via_stub_params(&mut self, params_and_value: u64) -> Option<u8> {
        let [_, _, _, _, _, _, flags, value] = params_and_value.to_be_bytes();

        self.interrupt = flags & SYS_VIA_STUB_FLAG_INTERRUPT != 0;

        self.trigger = if flags & SYS_VIA_STUB_FLAG_HAS_TRIGGER != 0 {
            Some(params_and_value >> 16)
        } else {
            None
        };

        if flags & SYS_VIA_STUB_FLAG_VALUE_IS_IC32 != 0 {
            self.ic32_latch.set(value);

            None
        } else {
            Some(value)
        }
    }
}

const SYS_VIA_STUB_FLAG_HAS_TRIGGER: u8 = 0x01;
const SYS_VIA_STUB_FLAG_INTERRUPT: u8 = 0x02;
const SYS_VIA_STUB_FLAG_VALUE_IS_IC32: u8 = 0x04;
