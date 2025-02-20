use crate::cycle_manager::*;

use super::registers::*;

pub struct Executor<'a, T>
where
    T: CycleManagerTrait + 'a,
{
    cycle_manager: &'a mut T,
    registers: &'a mut Registers,
}

impl<'a, T> Executor<'a, T>
where
    T: CycleManagerTrait + 'a,
{
    pub fn new(cycle_manager: &'a mut T, registers: &'a mut Registers) -> Self {
        Executor {
            cycle_manager,
            registers,
        }
    }

    pub fn execute(&mut self) -> Option<u8> {
        let opcode = self.read(self.registers.pc, CycleOp::None);
        self.inc_pc();

        match opcode {
            0x08 => {
                // PHP
                self.phantom_read(self.registers.pc);

                self.push(self.registers.get_p() | P_BIT_5_FLAG | P_BREAK_FLAG);
            }
            0x09 => {
                // ORA imm
                let val = self.read(self.registers.pc, CycleOp::None);
                self.inc_pc();

                self.or(val);
            }
            0x0a => {
                // ASL A
                self.phantom_read(self.registers.pc);

                self.registers.p_carry = (self.registers.a & 0x80) != 0;
                self.registers.a <<= 1;
                self.set_p_zero_negative(self.registers.a);
            }
            0x10 => {
                // BPL rel
                self.branch(!self.registers.p_negative);
            }
            0x20 => {
                // JSR abs
                let pc_low = self.read(self.registers.pc, CycleOp::None);
                self.inc_pc();

                self.stack_read();

                self.push_16(self.registers.pc);

                let pc_high = self.read(self.registers.pc, CycleOp::None);

                self.registers.pc = (pc_high as u16) << 8 | pc_low as u16;
            }
            0x26 => {
                // ROL zp
                let address = self.zpg_address();

                self.rol(address);
            }
            0x29 => {
                // AND imm
                let val = self.read(self.registers.pc, CycleOp::None);
                self.inc_pc();

                self.and(val);
            }
            0x48 => {
                // PHA
                self.phantom_read(self.registers.pc);

                self.push(self.registers.a);
            }
            0x4a => {
                // LSR A
                self.phantom_read(self.registers.pc);

                self.registers.p_carry = (self.registers.a & 0x01) > 0;

                self.registers.a = self.registers.a >> 1;
                self.registers.p_zero = self.registers.a == 0;
                self.registers.p_negative = false;
            }
            0x60 => {
                // RTS
                self.phantom_read(self.registers.pc);

                self.stack_read();

                self.registers.pc = self.pop_16();

                self.phantom_read(self.registers.pc);

                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
            0x66 => {
                // ROR zp
                let address = self.zpg_address();

                self.ror(address);
            }
            0x68 => {
                // PLA
                self.phantom_read(self.registers.pc);

                self.stack_read();

                self.registers.a = self.pop();
                self.set_p_zero_negative(self.registers.a);
            }
            0x6a => {
                // ROR A
                self.phantom_read(self.registers.pc);

                let old_val = self.registers.a;

                self.registers.a = (old_val >> 1) + (self.registers.p_carry as u8) * 0x80;

                self.set_p_zero_negative(self.registers.a);

                self.registers.p_carry = (old_val & 0x01) != 0;
            }
            0x78 => {
                // SEI
                self.phantom_read(self.registers.pc);

                self.registers.p_interrupt_disable = true;
            }
            0x85 => {
                // STA zp
                let address = self.zpg_address();

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0x86 => {
                // STX zp
                let address = self.zpg_address();

                self.write(address, self.registers.x, CycleOp::CheckInterrupt);
            }
            0x8a => {
                // TXA
                self.phantom_read(self.registers.pc);

                self.registers.a = self.registers.x;
                self.set_p_zero_negative(self.registers.a);
            }
            0x8c => {
                // STY abs
                let address = self.abs_address();

                self.write(address, self.registers.y, CycleOp::CheckInterrupt);
            }
            0x8d => {
                // STA abs
                let address = self.abs_address();

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0x8e => {
                // STX abs
                let address = self.abs_address();

                self.write(address, self.registers.x, CycleOp::CheckInterrupt);
            }
            0x91 => {
                // STA (zp),Y
                let address = self.ind_y_address();

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0x9a => {
                // TXS
                self.phantom_read(self.registers.pc);

                self.registers.s = self.registers.x;
            }
            0xa0 => {
                // LDY imm
                let val = self.read(self.registers.pc, CycleOp::None);
                self.inc_pc();

                self.ldy(val);
            }
            0xa2 => {
                // LDX imm
                let val = self.read(self.registers.pc, CycleOp::None);
                self.inc_pc();

                self.ldx(val);
            }
            0xa8 => {
                // TAY
                self.phantom_read(self.registers.pc);

                self.registers.y = self.registers.a;

                self.set_p_zero_negative(self.registers.a);
            }
            0xa9 => {
                // LDA imm
                let val = self.read(self.registers.pc, CycleOp::None);
                self.inc_pc();

                self.lda(val);
            }
            0xaa => {
                // TXA
                self.phantom_read(self.registers.pc);

                self.registers.x = self.registers.a;
                self.set_p_zero_negative(self.registers.a);
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
                self.branch(self.registers.p_carry);
            }
            0xca => {
                // DEX
                self.phantom_read(self.registers.pc);

                self.registers.x = self.registers.x.wrapping_sub(1);
                self.set_p_zero_negative(self.registers.x);
            }
            0xc5 => {
                // CMP zp
                let value = self.zpg_address_value();

                self.cmp(value);
            }
            0xc8 => {
                // INY
                self.phantom_read(self.registers.pc);

                self.registers.y = self.registers.y.wrapping_add(1);
                self.set_p_zero_negative(self.registers.y);
            }
            0xd0 => {
                // BNE rel
                self.branch(!self.registers.p_zero);
            }
            0xd8 => {
                // CLD
                self.phantom_read(self.registers.pc);

                self.registers.p_decimal_mode = false;
            }
            0xe0 => {
                // CPX imm
                let val = self.read(self.registers.pc, CycleOp::None);
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
                self.phantom_read(self.registers.pc);

                self.registers.x = self.registers.x.wrapping_add(1);
                self.set_p_zero_negative(self.registers.x);
            }
            0xf0 => {
                // BEQ rel
                self.branch(self.registers.p_zero);
            }
            _ => return Some(opcode),
        }

        self.cycle_manager.complete();

        None
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
        self.registers.set_p_zero_negative(in_operand);
    }

    fn read_u16_from_pc(&mut self) -> u16 {
        let low = self.read(self.registers.pc, CycleOp::None);
        self.inc_pc();
        let high = self.read(self.registers.pc, CycleOp::None);
        self.inc_pc();

        ((high as u16) << 8) | (low as u16)
    }

    fn inc_pc(&mut self) {
        self.registers.pc = self.registers.pc.wrapping_add(1);
    }

    fn push(&mut self, val: u8) {
        self.write(0x100 + (self.registers.s as u16), val, CycleOp::None);

        self.registers.s = self.registers.s.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.registers.s = self.registers.s.wrapping_add(1);

        self.read(0x100 + (self.registers.s as u16), CycleOp::None)
    }

    fn stack_read(&mut self) {
        self.read(0x100 + (self.registers.s as u16), CycleOp::None);
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
        let zero_page_address = self.read(self.registers.pc, CycleOp::None);
        self.inc_pc();

        zero_page_address as u16
    }

    fn zpg_address_value(&mut self) -> u8 {
        let address = self.zpg_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn ind_y_address(&mut self) -> u16 {
        let zpg_address = self.zpg_address();

        let low_address = self.read(zpg_address, CycleOp::None) as u16 + self.registers.y as u16;

        let high_address = self.read((zpg_address + 1) & 0xff, CycleOp::None) as u16;

        let address_without_carry = (high_address << 8) + (low_address & 0xff);

        self.phantom_read(address_without_carry);

        address_without_carry.wrapping_add(low_address & 0x100)
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.phantom_read(self.registers.pc);

            self.inc_pc();

            return;
        }

        let rel_address = self.read(self.registers.pc, CycleOp::None);

        self.inc_pc();

        self.phantom_read(self.registers.pc);

        let new_pc_low = (self.registers.pc & 0x00ff) + rel_address as u16;

        self.registers.pc = self.registers.pc & 0xff00 | new_pc_low & 0xff;

        let pc_high_adjustment =
            (new_pc_low & 0x100).wrapping_sub((rel_address as u16 & 0x80) << 1);

        if pc_high_adjustment != 0 {
            self.phantom_read(self.registers.pc);

            self.registers.pc = self.registers.pc.wrapping_add(pc_high_adjustment);
        }
    }

    fn cmp(&mut self, value: u8) {
        self.registers.p_carry = self.registers.a >= value;
        self.registers.p_zero = self.registers.a == value;
        self.registers.p_negative = self.registers.a.wrapping_sub(value) & 0x80 > 0;
    }

    fn rol(&mut self, address: u16) {
        let old_val = self.read(address, CycleOp::Sync);

        self.write(address, old_val, CycleOp::Sync);

        let new_val = (old_val << 1) + self.registers.p_carry as u8;

        self.write(address, new_val, CycleOp::Sync);

        self.registers.p_carry = (old_val & 0x80) > 0;
        self.set_p_zero_negative(new_val);
    }

    fn ror(&mut self, address: u16) {
        let old_val = self.read(address, CycleOp::Sync);

        self.write(address, old_val, CycleOp::Sync);

        let new_val = (old_val >> 1) + (self.registers.p_carry as u8) * 0x80;

        self.write(address, new_val, CycleOp::Sync);

        self.set_p_zero_negative(new_val);

        self.registers.p_carry = (old_val & 0x01) != 0;
    }

    fn cpx(&mut self, value: u8) {
        self.registers.p_carry = self.registers.x >= value;
        self.registers.p_zero = self.registers.x == value;
        self.registers.p_negative = self.registers.x.wrapping_sub(value) & 0x80 > 0;
    }

    fn lda(&mut self, operand: u8) {
        self.registers.a = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldx(&mut self, operand: u8) {
        self.registers.x = operand;
        self.set_p_zero_negative(operand);
    }

    fn ldy(&mut self, operand: u8) {
        self.registers.y = operand;
        self.set_p_zero_negative(operand);
    }

    fn and(&mut self, operand: u8) {
        self.registers.a &= operand;
        self.set_p_zero_negative(self.registers.a);
    }

    fn or(&mut self, operand: u8) {
        self.registers.a |= operand;
        self.set_p_zero_negative(self.registers.a);
    }
}
