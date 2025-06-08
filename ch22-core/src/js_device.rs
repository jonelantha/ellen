use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::device::Ch22Device;

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
        let read = Box::new(move |address: u16, machine_cycles: u32| {
            js_read
                .call2(&JsValue::NULL, &address.into(), &machine_cycles.into())
                .expect("js_read error")
                .as_f64()
                .expect("js_read error") as u8
        });

        let write = Box::new(move |address: u16, value: u8, machine_cycles: u32| {
            js_write
                .call3(
                    &JsValue::NULL,
                    &address.into(),
                    &value.into(),
                    &machine_cycles.into(),
                )
                .expect("js_write error");
        });

        let phase_2 = js_phase_2.map(|js_phase_2| {
            Box::new(move |machine_cycles: u32| {
                js_phase_2
                    .call1(&JsValue::NULL, &machine_cycles.into())
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

impl Ch22Device for JsCh22Device {
    fn read(&mut self, address: u16, machine_cycles: u32) -> u8 {
        (self.read)(address, machine_cycles)
    }

    fn write(&mut self, address: u16, value: u8, machine_cycles: u32) -> bool {
        (self.write)(address, value, machine_cycles);

        self.phase_2.is_some()
    }

    fn phase_2(&mut self, machine_cycles: u32) {
        if let Some(phase_2) = &self.phase_2 {
            (phase_2)(machine_cycles);
        }
    }

    fn is_slow(&self, _address: u16) -> bool {
        self.is_slow
    }
}
