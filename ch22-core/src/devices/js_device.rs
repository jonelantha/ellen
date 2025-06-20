use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::word::Word;

use super::io_device::Ch22IODevice;

pub struct JsCh22Device {
    read: Box<dyn Fn(u16, u32) -> u8>,
    write: Box<dyn Fn(u16, u8, u32)>,
    phase_2: Option<Box<dyn Fn(u32)>>,
    is_slow: bool,
}

impl JsCh22Device {
    pub fn new(
        js_read: Function,
        js_write: Function,
        js_phase_2: Option<Function>,
        is_slow: bool,
    ) -> JsCh22Device {
        let read = Box::new(move |address: u16, cycles: u32| {
            js_read
                .call2(&JsValue::NULL, &address.into(), &cycles.into())
                .expect("js_read error")
                .as_f64()
                .expect("js_read error") as u8
        });

        let write = Box::new(move |address: u16, value: u8, cycles: u32| {
            js_write
                .call3(
                    &JsValue::NULL,
                    &address.into(),
                    &value.into(),
                    &cycles.into(),
                )
                .expect("js_write error");
        });

        let phase_2 = js_phase_2.map(|js_phase_2| {
            Box::new(move |cycles: u32| {
                js_phase_2
                    .call1(&JsValue::NULL, &cycles.into())
                    .expect("js_phase_2 error");
            }) as Box<dyn Fn(u32)>
        });

        JsCh22Device {
            read,
            write,
            phase_2,
            is_slow,
        }
    }
}

impl Ch22IODevice for JsCh22Device {
    fn read(&mut self, address: Word, cycles: u32, _interrupt: &mut u8) -> u8 {
        (self.read)(address.into(), cycles)
    }

    fn write(&mut self, address: Word, value: u8, cycles: u32, _interrupt: &mut u8) -> bool {
        (self.write)(address.into(), value, cycles);

        self.phase_2.is_some()
    }

    fn phase_2(&mut self, _address: Word, cycles: u32, _interrupt: &mut u8) {
        if let Some(phase_2) = &self.phase_2 {
            (phase_2)(cycles);
        }
    }

    fn is_slow(&self) -> bool {
        self.is_slow
    }

    fn sync(&mut self, _cycles: u32, _interrupt: &mut u8) {}
}
