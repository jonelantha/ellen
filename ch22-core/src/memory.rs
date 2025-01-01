use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::utils;

const RAM_SIZE: usize = 0x10000;

#[wasm_bindgen]
pub struct Ch22Memory {
    whole_ram: [u8; RAM_SIZE],
    js_read_fallback: Function,
}

#[wasm_bindgen]
impl Ch22Memory {
    pub fn new(js_read_fallback: Function) -> Ch22Memory {
        utils::set_panic_hook();

        Ch22Memory {
            whole_ram: [0; RAM_SIZE],
            js_read_fallback,
        }
    }

    pub fn whole_ram_start(&self) -> *const u8 {
        self.whole_ram.as_ptr()
    }

    pub fn whole_ram_size(&self) -> usize {
        RAM_SIZE
    }

    pub fn read_direct(&self, address: usize) -> u8 {
        self.whole_ram[address]
    }

    pub fn write_direct(&mut self, address: usize, value: u8) {
        self.whole_ram[address] = value;
    }
}

impl Ch22Memory {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            ..0x8000 => self.whole_ram[address as usize],
            0x8000..0xc000 => self.js_read_fallback(address),
            0xc000..0xfc00 => self.whole_ram[address as usize],
            0xff00.. => self.whole_ram[address as usize],
            _ => panic!("not impl"),
        }
    }

    fn js_read_fallback(&self, address: u16) -> u8 {
        self.js_read_fallback
            .call1(&JsValue::NULL, &JsValue::from(address))
            .expect("js_read_fallback error")
            .as_f64()
            .expect("js_read_fallback error") as u8
    }
}
