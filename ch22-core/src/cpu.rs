use wasm_bindgen::prelude::*;

use crate::utils;

#[wasm_bindgen]
pub struct Ch22Cpu {
    pub pc: u16,
    pub p: u8,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new() -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu { pc: 0, p: 0 }
    }

    pub fn debug(&self) -> String {
        format!("PC: {:#06x}", self.pc)
    }

    pub fn set_p(
        &mut self,
        mask: u8,
        c: bool,
        z: bool,
        i: bool,
        d: bool,
        b: bool,
        v: bool,
        n: bool,
    ) {
        self.p &= !mask;
        self.p |= (c as u8)
            | ((z as u8) << 1)
            | ((i as u8) << 2)
            | ((d as u8) << 3)
            | ((b as u8) << 4)
            | ((v as u8) << 6)
            | ((n as u8) << 7);
    }
}

impl Default for Ch22Cpu {
    fn default() -> Self {
        Ch22Cpu::new()
    }
}
