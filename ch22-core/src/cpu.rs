use js_sys::Function;
use wasm_bindgen::prelude::*;
use web_sys::console;

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

    pub fn handle_advance_cycles(&self, cycles: u8, check_interrupt: bool) {
        self.js_advance_cycles
            .call2(&JsValue::NULL, &cycles.into(), &check_interrupt.into())
            .expect("js_advance_cycles error");
    }

    pub fn handle_instruction(
        &mut self,
        opcode: u8,
        memory: &mut Ch22Memory,
        cpuState: &mut Ch22CpuState,
    ) -> bool {
        let mut cycle_manager = CycleManager::new(
            memory,
            Box::new(|cycles, check_interrupt| self.handle_advance_cycles(cycles, check_interrupt)),
        );

        cpuState.handle_instruction(opcode, &mut cycle_manager)
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
impl Ch22CpuState {
    pub fn new() -> Ch22CpuState {
        Ch22CpuState {
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
}

impl Ch22CpuState {
    fn read_u16_from_pc(&mut self, bus_cycle_manager: &mut CycleManager) -> u16 {
        let low = bus_cycle_manager.read(self.pc, false, false);
        self.pc += 1;
        let high = bus_cycle_manager.read(self.pc, false, false);
        self.pc += 1;

        ((high as u16) << 8) | (low as u16)
    }

    fn abs_address(&mut self, bus_cycle_manager: &mut CycleManager) -> u16 {
        self.read_u16_from_pc(bus_cycle_manager)
    }

    fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.set_p_zero_negative(operand);
    }

    fn handle_instruction(&mut self, opcode: u8, cycle_manager: &mut CycleManager) -> bool {
        match opcode {
            0x8d => {
                // STA abs
                let address = self.abs_address(cycle_manager);

                cycle_manager.write(address, self.a, true, true);
            }
            0xa9 => {
                // LDA imm
                let val = cycle_manager.read(self.pc, false, false);
                self.pc += 1;

                self.lda(val);
            }
            _ => return false,
        }

        cycle_manager.complete();

        true
    }
}

pub struct CycleManager<'a> {
    pub cycles: u8,
    memory: &'a mut Ch22Memory,
    advance_cycles_handler: Box<dyn Fn(u8, bool) + 'a>,
}

impl<'a> CycleManager<'a> {
    fn new(memory: &'a mut Ch22Memory, advance_cycles_handler: Box<dyn Fn(u8, bool) + 'a>) -> Self {
        CycleManager {
            cycles: 1, // for opcode read
            memory,
            advance_cycles_handler,
        }
    }

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
