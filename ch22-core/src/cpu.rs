use wasm_bindgen::prelude::*;

use crate::utils;

const P_CARRY: u8 = 0b00000001;
const P_ZERO: u8 = 0b00000010;
const P_INTERUPT_DISABLE: u8 = 0b00000100;
const P_DECIMAL_MODE: u8 = 0b00001000;
const P_BREAK: u8 = 0b00010000;
const P_OVERFLOW: u8 = 0b01000000;
const P_NEGATIVE: u8 = 0b10000000;

#[wasm_bindgen]
pub struct Ch22Cpu {
    pub pc: u16,
    pub p: u8,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new() -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu {
            pc: 0,
            p: P_INTERUPT_DISABLE, // Interrupts off for starters
        }
    }

    pub fn debug(&self) -> String {
        format!("PC: {:#06x}", self.pc)
    }

    pub fn set_p_carry(&mut self, carry: bool) {
        if carry {
            self.p |= P_CARRY
        } else {
            self.p &= !P_CARRY
        }
    }

    pub fn get_p_carry(&self) -> bool {
        self.p & P_CARRY != 0
    }

    pub fn set_p_zero(&mut self, zero: bool) {
        if zero {
            self.p |= P_ZERO
        } else {
            self.p &= !P_ZERO
        }
    }

    pub fn get_p_zero(&self) -> bool {
        self.p & P_ZERO != 0
    }

    pub fn set_p_interrupt_disable(&mut self, interrupt_disable: bool) {
        if interrupt_disable {
            self.p |= P_INTERUPT_DISABLE
        } else {
            self.p &= !P_INTERUPT_DISABLE
        }
    }

    pub fn get_p_interrupt_disable(&self) -> bool {
        self.p & P_INTERUPT_DISABLE != 0
    }

    pub fn set_p_decimal_mode(&mut self, decimal_mode: bool) {
        if decimal_mode {
            self.p |= P_DECIMAL_MODE
        } else {
            self.p &= !P_DECIMAL_MODE
        }
    }

    pub fn get_p_decimal_mode(&self) -> bool {
        self.p & P_DECIMAL_MODE != 0
    }

    pub fn set_p_break(&mut self, brk: bool) {
        if brk {
            self.p |= P_BREAK
        } else {
            self.p &= !P_BREAK
        }
    }

    pub fn get_p_break(&self) -> bool {
        self.p & P_BREAK != 0
    }

    pub fn set_p_overflow(&mut self, overflow: bool) {
        if overflow {
            self.p |= P_OVERFLOW
        } else {
            self.p &= !P_OVERFLOW
        }
    }

    pub fn get_p_overflow(&self) -> bool {
        self.p & P_OVERFLOW != 0
    }

    pub fn set_p_negative(&mut self, negative: bool) {
        if negative {
            self.p |= P_NEGATIVE
        } else {
            self.p &= !P_NEGATIVE
        }
    }

    pub fn get_p_negative(&self) -> bool {
        self.p & P_NEGATIVE != 0
    }

    pub fn set_p_zero_negative(&mut self, in_operand: u8) {
        self.set_p_zero(in_operand == 0);
        self.set_p_negative((in_operand & 0b10000000) != 0);
    }
}

impl Default for Ch22Cpu {
    fn default() -> Self {
        Ch22Cpu::new()
    }
}
