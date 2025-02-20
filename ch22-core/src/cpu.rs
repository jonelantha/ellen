use js_sys::Function;
use wasm_bindgen::prelude::*;
//use web_sys::console;

use crate::cycle_manager::*;
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

        let mut executor = Executor::new(&mut cycle_manager, cpu_state);

        executor.execute()
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

pub struct Executor<'a, T>
where
    T: CycleManagerTrait + 'a,
{
    cycle_manager: &'a mut T,
    cpu_state: &'a mut Ch22CpuState,
}

impl<'a, T> Executor<'a, T>
where
    T: CycleManagerTrait + 'a,
{
    pub fn new(cycle_manager: &'a mut T, cpu_state: &'a mut Ch22CpuState) -> Self {
        Executor {
            cycle_manager,
            cpu_state,
        }
    }

    fn phantom_read(&mut self, address: u16) {
        self.cycle_manager.phantom_read(address);
    }

    fn read(&mut self, address: u16, op: CycleOp) -> u8 {
        self.cycle_manager.read(address, op)
    }

    fn write(&mut self, address: u16, value: u8, op: CycleOp) {
        self.cycle_manager.write(address, value, op);
    }

    fn set_p_zero_negative(&mut self, in_operand: u8) {
        self.cpu_state.set_p_zero_negative(in_operand);
    }

    fn read_u16_from_pc(&mut self) -> u16 {
        let low = self.read(self.cpu_state.pc, CycleOp::None);
        self.inc_pc();
        let high = self.read(self.cpu_state.pc, CycleOp::None);
        self.inc_pc();

        ((high as u16) << 8) | (low as u16)
    }

    fn inc_pc(&mut self) {
        self.cpu_state.pc = self.cpu_state.pc.wrapping_add(1);
    }

    fn push(&mut self, val: u8) {
        self.write(0x100 + (self.cpu_state.s as u16), val, CycleOp::None);

        self.cpu_state.s = self.cpu_state.s.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.cpu_state.s = self.cpu_state.s.wrapping_add(1);

        self.read(0x100 + (self.cpu_state.s as u16), CycleOp::None)
    }

    fn stack_read(&mut self) {
        self.read(0x100 + (self.cpu_state.s as u16), CycleOp::None);
    }

    fn push_16(&mut self, val: u16) {
        self.push((val >> 8) as u8);
        self.push((val & 0xff) as u8);
    }

    fn pop_16(&mut self) -> u16 {
        let low = self.pop();
        let high = self.pop();

        ((high as u16) << 8) | (low as u16)
    }

    fn abs_address(&mut self) -> u16 {
        self.read_u16_from_pc()
    }

    fn abs_address_value(&mut self) -> u8 {
        let address = self.abs_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn zpg_address(&mut self) -> u16 {
        let zero_page_address = self.read(self.cpu_state.pc, CycleOp::None);
        self.inc_pc();

        zero_page_address as u16
    }

    fn zpg_address_value(&mut self) -> u8 {
        let address = self.zpg_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn ind_y_address(&mut self) -> u16 {
        let zpg_address = self.zpg_address();

        let low_address = self.read(zpg_address, CycleOp::None) as u16 + self.cpu_state.y as u16;

        let high_address = self.read((zpg_address + 1) & 0xff, CycleOp::None) as u16;

        let address_without_carry = (high_address << 8) + (low_address & 0xff);

        self.phantom_read(address_without_carry);

        address_without_carry.wrapping_add(low_address & 0x100)
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.phantom_read(self.cpu_state.pc);

            self.inc_pc();

            return;
        }

        let rel_address = self.read(self.cpu_state.pc, CycleOp::None);

        self.inc_pc();

        self.phantom_read(self.cpu_state.pc);

        let new_pc_low = (self.cpu_state.pc & 0x00ff) + rel_address as u16;

        self.cpu_state.pc = self.cpu_state.pc & 0xff00 | new_pc_low & 0xff;

        let pc_high_adjustment =
            (new_pc_low & 0x100).wrapping_sub((rel_address as u16 & 0x80) << 1);

        if pc_high_adjustment != 0 {
            self.phantom_read(self.cpu_state.pc);

            self.cpu_state.pc = self.cpu_state.pc.wrapping_add(pc_high_adjustment);
        }
    }

    fn cmp(&mut self, value: u8) {
        self.cpu_state.p_carry = self.cpu_state.a >= value;
        self.cpu_state.p_zero = self.cpu_state.a == value;
        self.cpu_state.p_negative = self.cpu_state.a.wrapping_sub(value) & 0x80 > 0;
    }

    fn rol(&mut self, address: u16) {
        let old_val = self.read(address, CycleOp::Sync);

        self.write(address, old_val, CycleOp::Sync);

        let new_val = (old_val << 1) + self.cpu_state.p_carry as u8;

        self.write(address, new_val, CycleOp::Sync);

        self.cpu_state.p_carry = (old_val & 0x80) > 0;
        self.set_p_zero_negative(new_val);
    }

    fn ror(&mut self, address: u16) {
        let old_val = self.read(address, CycleOp::Sync);

        self.write(address, old_val, CycleOp::Sync);

        let new_val = (old_val >> 1) + (self.cpu_state.p_carry as u8) * 0x80;

        self.write(address, new_val, CycleOp::Sync);

        self.set_p_zero_negative(new_val);

        self.cpu_state.p_carry = (old_val & 0x01) != 0;
    }

    fn cpx(&mut self, value: u8) {
        self.cpu_state.p_carry = self.cpu_state.x >= value;
        self.cpu_state.p_zero = self.cpu_state.x == value;
        self.cpu_state.p_negative = self.cpu_state.x.wrapping_sub(value) & 0x80 > 0;
    }

    fn lda(&mut self, operand: u8) {
        self.cpu_state.a = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldx(&mut self, operand: u8) {
        self.cpu_state.x = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldy(&mut self, operand: u8) {
        self.cpu_state.y = operand;
        self.set_p_zero_negative(operand);
    }

    fn and(&mut self, operand: u8) {
        self.cpu_state.a &= operand;
        self.set_p_zero_negative(self.cpu_state.a);
    }

    fn or(&mut self, operand: u8) {
        self.cpu_state.a |= operand;
        self.set_p_zero_negative(self.cpu_state.a);
    }

    pub fn execute(&mut self) -> Option<u8> {
        let opcode = self.read(self.cpu_state.pc, CycleOp::None);
        self.inc_pc();

        match opcode {
            0x08 => {
                // PHP
                self.phantom_read(self.cpu_state.pc);

                self.push(self.cpu_state.get_p() | P_BIT_5_FLAG | P_BREAK_FLAG);
            }
            0x09 => {
                // ORA imm
                let val = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.or(val);
            }
            0x0a => {
                // ASL A
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.p_carry = (self.cpu_state.a & 0x80) != 0;
                self.cpu_state.a <<= 1;
                self.set_p_zero_negative(self.cpu_state.a);
            }
            0x10 => {
                // BPL rel
                self.branch(!self.cpu_state.p_negative);
            }
            0x20 => {
                // JSR abs
                let pc_low = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.stack_read();

                self.push_16(self.cpu_state.pc);

                let pc_high = self.read(self.cpu_state.pc, CycleOp::None);

                self.cpu_state.pc = (pc_high as u16) << 8 | pc_low as u16;
            }
            0x26 => {
                // ROL zp
                let address = self.zpg_address();

                self.rol(address);
            }
            0x29 => {
                // AND imm
                let val = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.and(val);
            }
            0x48 => {
                // PHA
                self.phantom_read(self.cpu_state.pc);

                self.push(self.cpu_state.a);
            }
            0x4a => {
                // LSR A
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.p_carry = (self.cpu_state.a & 0x01) > 0;

                self.cpu_state.a = self.cpu_state.a >> 1;
                self.cpu_state.p_zero = self.cpu_state.a == 0;
                self.cpu_state.p_negative = false;
            }
            0x60 => {
                // RTS
                self.phantom_read(self.cpu_state.pc);

                self.stack_read();

                self.cpu_state.pc = self.pop_16();

                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.pc = self.cpu_state.pc.wrapping_add(1);
            }
            0x66 => {
                // ROR zp
                let address = self.zpg_address();

                self.ror(address);
            }
            0x68 => {
                // PLA
                self.phantom_read(self.cpu_state.pc);

                self.stack_read();

                self.cpu_state.a = self.pop();
                self.set_p_zero_negative(self.cpu_state.a);
            }
            0x6a => {
                // ROR A
                self.phantom_read(self.cpu_state.pc);

                let old_val = self.cpu_state.a;

                self.cpu_state.a = (old_val >> 1) + (self.cpu_state.p_carry as u8) * 0x80;

                self.set_p_zero_negative(self.cpu_state.a);

                self.cpu_state.p_carry = (old_val & 0x01) != 0;
            }
            0x78 => {
                // SEI
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.p_interrupt_disable = true;
            }
            0x85 => {
                // STA zp
                let address = self.zpg_address();

                self.write(address, self.cpu_state.a, CycleOp::CheckInterrupt);
            }
            0x86 => {
                // STX zp
                let address = self.zpg_address();

                self.write(address, self.cpu_state.x, CycleOp::CheckInterrupt);
            }
            0x8a => {
                // TXA
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.a = self.cpu_state.x;
                self.set_p_zero_negative(self.cpu_state.a);
            }
            0x8c => {
                // STY abs
                let address = self.abs_address();

                self.write(address, self.cpu_state.y, CycleOp::CheckInterrupt);
            }
            0x8d => {
                // STA abs
                let address = self.abs_address();

                self.write(address, self.cpu_state.a, CycleOp::CheckInterrupt);
            }
            0x8e => {
                // STX abs
                let address = self.abs_address();

                self.write(address, self.cpu_state.x, CycleOp::CheckInterrupt);
            }
            0x91 => {
                // STA (zp),Y
                let address = self.ind_y_address();

                self.write(address, self.cpu_state.a, CycleOp::CheckInterrupt);
            }
            0x9a => {
                // TXS
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.s = self.cpu_state.x;
            }
            0xa0 => {
                // LDY imm
                let val = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.ldy(val);
            }
            0xa2 => {
                // LDX imm
                let val = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.ldx(val);
            }
            0xa8 => {
                // TAY
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.y = self.cpu_state.a;

                self.set_p_zero_negative(self.cpu_state.a);
            }
            0xa9 => {
                // LDA imm
                let val = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.lda(val);
            }
            0xaa => {
                // TXA
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.x = self.cpu_state.a;
                self.set_p_zero_negative(self.cpu_state.a);
            }
            0xad => {
                // LDA abs
                let value = self.abs_address_value();

                self.lda(value);
            }
            0xae => {
                // LDX abs
                let value = self.abs_address_value();

                self.ldx(value);
            }
            0xb0 => {
                // BCS rel
                self.branch(self.cpu_state.p_carry);
            }
            0xca => {
                // DEX
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.x = self.cpu_state.x.wrapping_sub(1);
                self.set_p_zero_negative(self.cpu_state.x);
            }
            0xc5 => {
                // CMP zp
                let value = self.zpg_address_value();

                self.cmp(value);
            }
            0xc8 => {
                // INY
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.y = self.cpu_state.y.wrapping_add(1);
                self.set_p_zero_negative(self.cpu_state.y);
            }
            0xd0 => {
                // BNE rel
                self.branch(!self.cpu_state.p_zero);
            }
            0xd8 => {
                // CLD
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.p_decimal_mode = false;
            }
            0xe0 => {
                // CPX imm
                let val = self.read(self.cpu_state.pc, CycleOp::None);
                self.inc_pc();

                self.cpx(val);
            }
            0xe6 => {
                // INC zp
                let address = self.zpg_address();

                let mut value = self.read(address, CycleOp::Sync);

                self.write(address, value, CycleOp::Sync);

                value = value.wrapping_add(1);

                self.write(address, value, CycleOp::Sync);

                self.set_p_zero_negative(value);
            }
            0xe8 => {
                // INX
                self.phantom_read(self.cpu_state.pc);

                self.cpu_state.x = self.cpu_state.x.wrapping_add(1);
                self.set_p_zero_negative(self.cpu_state.x);
            }
            0xf0 => {
                // BEQ rel
                self.branch(self.cpu_state.p_zero);
            }
            _ => return Some(opcode),
        }

        self.cycle_manager.complete();

        None
    }
}
