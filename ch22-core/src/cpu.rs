use js_sys::Function;
use wasm_bindgen::prelude::*;
//use web_sys::console;

use crate::memory::*;
use crate::utils;

#[wasm_bindgen]
pub struct Ch22Cpu {
    js_advance_cycles: Function,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new(js_advance_cycles: Function) -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu { js_advance_cycles }
    }

    fn handle_advance_cycles(&self, cycles: u8, check_interrupt: bool) {
        self.js_advance_cycles
            .call2(&JsValue::NULL, &cycles.into(), &check_interrupt.into())
            .expect("js_advance_cycles error");
    }

    pub fn handle_next_instruction(
        &mut self,
        memory: &mut Ch22Memory,
        cpu_state: &mut Ch22CpuState,
    ) -> Option<u8> {
        let mut cycle_manager = CycleManager::new(
            memory,
            Box::new(|cycles, check_interrupt| self.handle_advance_cycles(cycles, check_interrupt)),
        );

        cpu_state.handle_next_instruction(&mut cycle_manager)
    }
}

const P_CARRY_FLAG: u8 = 0b00000001;
const P_ZERO_FLAG: u8 = 0b00000010;
const P_I_INTERRUPT_DISABLE_FLAG: u8 = 0b00000100;
const P_DECIMAL_MODE_FLAG: u8 = 0b00001000;
const P_BREAK_FLAG: u8 = 0b00010000;
const P_BIT_5_FLAG: u8 = 0b00100000;
const P_OVERFLOW_FLAG: u8 = 0b01000000;
const P_NEGATIVE_FLAG: u8 = 0b10000000;

#[wasm_bindgen]
pub struct Ch22CpuState {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
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
impl Ch22CpuState {
    pub fn new() -> Ch22CpuState {
        Ch22CpuState {
            pc: 0,
            s: 0xff,
            a: 0,
            x: 0,
            y: 0,
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
}

impl Default for Ch22CpuState {
    fn default() -> Self {
        Self::new()
    }
}

impl Ch22CpuState {
    fn read_u16_from_pc(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u16 {
        let low = cycle_manager.read(self.pc, false, false);
        self.inc_pc();
        let high = cycle_manager.read(self.pc, false, false);
        self.inc_pc();

        ((high as u16) << 8) | (low as u16)
    }

    fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    fn abs_address(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u16 {
        self.read_u16_from_pc(cycle_manager)
    }

    fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldx(&mut self, operand: u8) {
        self.x = operand;
        self.set_p_zero_negative(operand);
    }

    pub fn handle_next_instruction(
        &mut self,
        cycle_manager: &mut impl CycleManagerTrait,
    ) -> Option<u8> {
        let opcode = cycle_manager.read(self.pc, false, false);
        self.inc_pc();

        match opcode {
            0x78 => {
                // SEI
                cycle_manager.read(self.pc, false, false);

                self.p_interrupt_disable = true;
            }
            0x8d => {
                // STA abs
                let address = self.abs_address(cycle_manager);

                cycle_manager.write(address, self.a, true, true);
            }
            0x9a => {
                // TXS
                cycle_manager.read(self.pc, false, false);

                self.s = self.x;
            }
            0xa2 => {
                // LDX imm
                let val = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.ldx(val);
            }
            0xa9 => {
                // LDA imm
                let val = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.lda(val);
            }
            0xd8 => {
                // CLD
                cycle_manager.read(self.pc, false, false);

                self.p_decimal_mode = false;
            }
            _ => return Some(opcode),
        }

        cycle_manager.complete();

        None
    }
}

pub trait CycleManagerTrait {
    fn read(&mut self, address: u16, sync: bool, check_interrupt: bool) -> u8;
    fn write(&mut self, address: u16, value: u8, sync: bool, check_interrupt: bool);
    fn complete(&self);
}

struct CycleManager<'a> {
    cycles: u8,
    memory: &'a mut Ch22Memory,
    advance_cycles_handler: Box<dyn Fn(u8, bool) + 'a>,
}

impl<'a> CycleManager<'a> {
    fn new(memory: &'a mut Ch22Memory, advance_cycles_handler: Box<dyn Fn(u8, bool) + 'a>) -> Self {
        CycleManager {
            cycles: 0,
            memory,
            advance_cycles_handler,
        }
    }
}

impl CycleManagerTrait for CycleManager<'_> {
    fn read(&mut self, address: u16, sync: bool, check_interrupt: bool) -> u8 {
        if sync {
            (self.advance_cycles_handler)(self.cycles, check_interrupt);

            self.cycles = 0;
        }

        let value = self.memory.read(address);

        self.cycles += 1;

        //console::log_1(&format!("read {:x} {:x}", address, value).into());

        value
    }

    fn write(&mut self, address: u16, value: u8, sync: bool, check_interrupt: bool) {
        if sync {
            (self.advance_cycles_handler)(self.cycles, check_interrupt);

            self.cycles = 0;
        }
        //console::log_1(&format!("write {:x} {:x}", address, value).into());

        self.memory.write(address, value);

        self.cycles += 1;
    }

    fn complete(&self) {
        (self.advance_cycles_handler)(self.cycles, false);
        //console::log_1(&format!("complete {:x}", self.cycles).into());
    }
}
