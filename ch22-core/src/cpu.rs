use wasm_bindgen::prelude::*;

use crate::memory::*;
use crate::utils;

const P_CARRY_FLAG: u8 = 0b00000001;
const P_ZERO_FLAG: u8 = 0b00000010;
const P_I_INTERRUPT_DISABLE_FLAG: u8 = 0b00000100;
const P_DECIMAL_MODE_FLAG: u8 = 0b00001000;
const P_BREAK_FLAG: u8 = 0b00010000;
const P_BIT_5_FLAG: u8 = 0b00100000;
const P_OVERFLOW_FLAG: u8 = 0b01000000;
const P_NEGATIVE_FLAG: u8 = 0b10000000;

#[wasm_bindgen]
pub struct Ch22Cpu {
    pub pc: u16,
    pub a: u8,
    pub p_carry: bool,
    pub p_zero: bool,
    pub p_interrupt_disable: bool,
    pub p_decimal_mode: bool,
    pub p_break: bool,
    pub p_bit_5: bool,
    pub p_overflow: bool,
    pub p_negative: bool,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new() -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu {
            pc: 0,
            a: 0,
            p_carry: false,
            p_zero: false,
            p_interrupt_disable: true,
            p_decimal_mode: false,
            p_break: false,
            p_bit_5: false,
            p_overflow: false,
            p_negative: false,
        }
    }

    pub fn get_p(&self) -> u8 {
        (if self.p_carry { P_CARRY_FLAG } else { 0 })
            | (if self.p_zero { P_ZERO_FLAG } else { 0 })
            | (if self.p_interrupt_disable {
                P_I_INTERRUPT_DISABLE_FLAG
            } else {
                0
            })
            | (if self.p_decimal_mode {
                P_DECIMAL_MODE_FLAG
            } else {
                0
            })
            | (if self.p_break { P_BREAK_FLAG } else { 0 })
            | (if self.p_bit_5 { P_BIT_5_FLAG } else { 0 })
            | (if self.p_overflow { P_OVERFLOW_FLAG } else { 0 })
            | (if self.p_negative { P_NEGATIVE_FLAG } else { 0 })
    }

    pub fn set_p(&mut self, p: u8) {
        self.p_carry = p & P_CARRY_FLAG != 0;
        self.p_zero = p & P_ZERO_FLAG != 0;
        self.p_interrupt_disable = p & P_I_INTERRUPT_DISABLE_FLAG != 0;
        self.p_decimal_mode = p & P_DECIMAL_MODE_FLAG != 0;
        self.p_break = p & P_BREAK_FLAG != 0;
        self.p_bit_5 = p & P_BIT_5_FLAG != 0;
        self.p_overflow = p & P_OVERFLOW_FLAG != 0;
        self.p_negative = p & P_NEGATIVE_FLAG != 0;
    }

    pub fn set_p_zero_negative(&mut self, in_operand: u8) {
        self.p_zero = in_operand == 0;
        self.p_negative = (in_operand & 0b10000000) != 0;
    }

    pub fn handle_instruction(&mut self, opcode: u8, memory: &Ch22Memory) -> bool {
        let mut handled = true;

        match opcode {
            0xa9 => {
                // LDA imm
                self.lda(memory.read(self.pc));
                self.pc += 1;
            }
            _ => {
                handled = false;
            }
        }

        handled
    }
}

impl Ch22Cpu {
    fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.set_p_zero_negative(operand);
    }
}

impl Default for Ch22Cpu {
    fn default() -> Self {
        Ch22Cpu::new()
    }
}
