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

    fn push(&mut self, cycle_manager: &mut impl CycleManagerTrait, val: u8) {
        cycle_manager.write(0x100 + (self.s as u16), val, false, false);

        self.s = self.s.wrapping_sub(1);
    }

    fn pop(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u8 {
        self.s = self.s.wrapping_add(1);

        cycle_manager.read(0x100 + (self.s as u16), false, false)
    }

    fn stack_read(&mut self, cycle_manager: &mut impl CycleManagerTrait) {
        cycle_manager.read(0x100 + (self.s as u16), false, false);
    }

    fn push_16(&mut self, cycle_manager: &mut impl CycleManagerTrait, val: u16) {
        self.push(cycle_manager, (val >> 8) as u8);
        self.push(cycle_manager, (val & 0xff) as u8);
    }

    fn pop_16(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u16 {
        let low = self.pop(cycle_manager);
        let high = self.pop(cycle_manager);

        ((high as u16) << 8) | (low as u16)
    }

    fn abs_address(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u16 {
        self.read_u16_from_pc(cycle_manager)
    }

    fn abs_address_value(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u8 {
        let address = self.abs_address(cycle_manager);

        cycle_manager.read(address, true, true)
    }

    fn zpg_address(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u16 {
        let zero_page_address = cycle_manager.read(self.pc, false, false);
        self.inc_pc();

        zero_page_address as u16
    }

    fn zpg_address_value(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u8 {
        let address = self.zpg_address(cycle_manager);

        cycle_manager.read(address, true, true)
    }

    fn ind_y_address(&mut self, cycle_manager: &mut impl CycleManagerTrait) -> u16 {
        let zpg_address = self.zpg_address(cycle_manager);

        let low_address = cycle_manager.read(zpg_address, false, false) as u16 + self.y as u16;

        let high_address = cycle_manager.read((zpg_address + 1) & 0xff, false, false) as u16;

        let address_without_carry = (high_address << 8) + (low_address & 0xff);

        cycle_manager.read(address_without_carry, false, false);

        address_without_carry.wrapping_add(low_address & 0x100)
    }

    fn branch(&mut self, cycle_manager: &mut impl CycleManagerTrait, condition: bool) {
        if !condition {
            cycle_manager.read(self.pc, false, false);

            self.inc_pc();

            return;
        }

        let rel_address = cycle_manager.read(self.pc, false, false);

        self.inc_pc();

        cycle_manager.read(self.pc, false, false);

        let new_pc_low = (self.pc & 0x00ff) + rel_address as u16;

        self.pc = self.pc & 0xff00 | new_pc_low & 0xff;

        let pc_high_adjustment =
            (new_pc_low & 0x100).wrapping_sub((rel_address as u16 & 0x80) << 1);

        if pc_high_adjustment != 0 {
            cycle_manager.read(self.pc, false, false);

            self.pc = self.pc.wrapping_add(pc_high_adjustment);
        }
    }

    fn cmp(&mut self, value: u8) {
        self.p_carry = self.a >= value;
        self.p_zero = self.a == value;
        self.p_negative = self.a.wrapping_sub(value) & 0x80 > 0;
    }

    fn rol(&mut self, cycle_manager: &mut impl CycleManagerTrait, address: u16) {
        let old_val = cycle_manager.read(address, true, false);

        cycle_manager.write(address, old_val, true, false);

        let new_val = (old_val << 1) + self.p_carry as u8;

        cycle_manager.write(address, new_val, true, false);

        self.p_carry = (old_val & 0x80) > 0;
        self.set_p_zero_negative(new_val);
    }

    fn ror(&mut self, cycle_manager: &mut impl CycleManagerTrait, address: u16) {
        let old_val = cycle_manager.read(address, true, false);

        cycle_manager.write(address, old_val, true, false);

        let new_val = (old_val >> 1) + (self.p_carry as u8) * 0x80;

        cycle_manager.write(address, new_val, true, false);

        self.set_p_zero_negative(new_val);

        self.p_carry = (old_val & 0x01) != 0;
    }

    fn cpx(&mut self, value: u8) {
        self.p_carry = self.x >= value;
        self.p_zero = self.x == value;
        self.p_negative = self.x.wrapping_sub(value) & 0x80 > 0;
    }

    fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldx(&mut self, operand: u8) {
        self.x = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldy(&mut self, operand: u8) {
        self.y = operand;
        self.set_p_zero_negative(operand);
    }

    pub fn handle_next_instruction(
        &mut self,
        cycle_manager: &mut impl CycleManagerTrait,
    ) -> Option<u8> {
        let opcode = cycle_manager.read(self.pc, false, false);
        self.inc_pc();

        match opcode {
            0x08 => {
                // PHP
                cycle_manager.read(self.pc, false, false);

                self.push(cycle_manager, self.get_p() | 0x10 | 0x20);
            }
            0x0a => {
                // ASL A
                cycle_manager.read(self.pc, false, false);

                self.p_carry = (self.a & 0x80) != 0;
                self.a <<= 1;
                self.set_p_zero_negative(self.a);
            }
            0x10 => {
                // BPL rel
                self.branch(cycle_manager, !self.p_negative);
            }
            0x20 => {
                // JSR abs
                let pc_low = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.stack_read(cycle_manager);

                self.push_16(cycle_manager, self.pc);

                let pc_high = cycle_manager.read(self.pc, false, false);

                self.pc = (pc_high as u16) << 8 | pc_low as u16;
            }
            0x26 => {
                // ROL zp

                let address = self.zpg_address(cycle_manager);

                self.rol(cycle_manager, address);
            }
            0x48 => {
                // PHA
                cycle_manager.read(self.pc, false, false);

                self.push(cycle_manager, self.a);
            }
            0x60 => {
                // RTS
                cycle_manager.read(self.pc, false, false);

                self.stack_read(cycle_manager);

                self.pc = self.pop_16(cycle_manager);

                cycle_manager.read(self.pc, false, false);

                self.pc = self.pc.wrapping_add(1);
            }
            0x66 => {
                // ROR zp
                let address = self.zpg_address(cycle_manager);

                self.ror(cycle_manager, address);
            }
            0x78 => {
                // SEI
                cycle_manager.read(self.pc, false, false);

                self.p_interrupt_disable = true;
            }
            0x85 => {
                // STA zp
                let address = self.zpg_address(cycle_manager);

                cycle_manager.write(address, self.a, true, true);
            }
            0x86 => {
                // STX zp
                let address = self.zpg_address(cycle_manager);

                cycle_manager.write(address, self.x, true, true);
            }
            0x8a => {
                // TXA
                cycle_manager.read(self.pc, false, false);

                self.a = self.x;
                self.set_p_zero_negative(self.a);
            }
            0x8c => {
                // STY abs
                let address = self.abs_address(cycle_manager);

                cycle_manager.write(address, self.y, true, true);
            }
            0x8d => {
                // STA abs
                let address = self.abs_address(cycle_manager);

                cycle_manager.write(address, self.a, true, true);
            }
            0x8e => {
                // STX abs
                let address = self.abs_address(cycle_manager);

                cycle_manager.write(address, self.x, true, true);
            }
            0x91 => {
                // STA (zp),Y
                let address = self.ind_y_address(cycle_manager);

                cycle_manager.write(address, self.a, true, true);
            }
            0x9a => {
                // TXS
                cycle_manager.read(self.pc, false, false);

                self.s = self.x;
            }
            0xa0 => {
                // LDY imm
                let val = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.ldy(val);
            }
            0xa2 => {
                // LDX imm
                let val = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.ldx(val);
            }
            0xa8 => {
                // TAY
                cycle_manager.read(self.pc, false, false);

                self.y = self.a;

                self.set_p_zero_negative(self.a);
            }
            0xa9 => {
                // LDA imm
                let val = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.lda(val);
            }
            0xaa => {
                // TXA
                cycle_manager.read(self.pc, false, false);

                self.x = self.a;
                self.set_p_zero_negative(self.a);
            }
            0xad => {
                // LDA abs
                let value = self.abs_address_value(cycle_manager);

                self.lda(value);
            }
            0xae => {
                // LDX abs
                let value = self.abs_address_value(cycle_manager);

                self.ldx(value);
            }
            0xb0 => {
                // BCS rel
                self.branch(cycle_manager, self.p_carry);
            }
            0xca => {
                // DEX
                cycle_manager.read(self.pc, false, false);

                self.x = self.x.wrapping_sub(1);
                self.set_p_zero_negative(self.x);
            }
            0xc5 => {
                // CMP zp
                let value = self.zpg_address_value(cycle_manager);

                self.cmp(value);
            }
            0xc8 => {
                // INY
                cycle_manager.read(self.pc, false, false);

                self.y = self.y.wrapping_add(1);
                self.set_p_zero_negative(self.y);
            }
            0xd0 => {
                // BNE rel
                self.branch(cycle_manager, !self.p_zero);
            }
            0xd8 => {
                // CLD
                cycle_manager.read(self.pc, false, false);

                self.p_decimal_mode = false;
            }
            0xe0 => {
                // CPX imm
                let val = cycle_manager.read(self.pc, false, false);
                self.inc_pc();

                self.cpx(val);
            }
            0xe6 => {
                // INC zp
                let address = self.zpg_address(cycle_manager);

                let mut value = cycle_manager.read(address, true, false);

                cycle_manager.write(address, value, true, false);

                value = value.wrapping_add(1);

                cycle_manager.write(address, value, true, false);

                self.set_p_zero_negative(value);
            }
            0xe8 => {
                // INX
                cycle_manager.read(self.pc, false, false);

                self.x = self.x.wrapping_add(1);
                self.set_p_zero_negative(self.x);
            }
            0xf0 => {
                // BEQ rel
                self.branch(cycle_manager, self.p_zero);
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
