use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::utils;

const RAM_SIZE: usize = 0x10000;

#[wasm_bindgen]
pub struct Ch22Memory {
    ram: [u8; RAM_SIZE],
    js_read_fallback: Function,
    js_write_fallback: Function,
}

#[wasm_bindgen]
impl Ch22Memory {
    pub fn new(js_read_fallback: Function, js_write_fallback: Function) -> Ch22Memory {
        utils::set_panic_hook();

        Ch22Memory {
            ram: [0; RAM_SIZE],
            js_read_fallback,
            js_write_fallback,
        }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn ram_size(&self) -> usize {
        RAM_SIZE
    }
}

impl Ch22Memory {
    pub fn read(&self, address: u16, cycles: u8) -> u8 {
        match address {
            ..0x8000 => self.ram[address as usize],
            0x8000..0xc000 => self.js_read_fallback(address, cycles),
            0xc000..0xfc00 => self.ram[address as usize],
            0xfc00..0xff00 => self.js_read_fallback(address, cycles),
            0xff00.. => self.ram[address as usize],
        }
    }

    pub fn write(&mut self, address: u16, value: u8, cycles: u8) -> bool {
        match address {
            ..0x8000 => {
                self.ram[address as usize] = value;
                false
            }
            0x8000..0xc000 => false,
            _ => self.js_write_fallback(address, value, cycles),
        }
    }

    fn js_read_fallback(&self, address: u16, cycles: u8) -> u8 {
        self.js_read_fallback
            .call2(
                &JsValue::NULL,
                &JsValue::from(address),
                &JsValue::from(cycles),
            )
            .expect("js_read_fallback error")
            .as_f64()
            .expect("js_read_fallback error") as u8
    }

    fn js_write_fallback(&self, address: u16, value: u8, cycles: u8) -> bool {
        self.js_write_fallback
            .call3(
                &JsValue::NULL,
                &JsValue::from(address),
                &JsValue::from(value),
                &JsValue::from(cycles),
            )
            .expect("js_write_fallback error")
            .as_bool()
            .expect("js_write_fallback error")
    }
}
