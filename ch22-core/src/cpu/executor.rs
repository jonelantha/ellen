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
        let opcode = self.imm();

        match opcode {
            0x08 => {
                // PHP
                self.phantom_pc_read();

                self.push(self.registers.get_p() | P_BIT_5_FLAG | P_BREAK_FLAG);
            }
            0x09 => {
                // ORA imm
                let value = self.imm();

                self.or(value);
            }
            0x0a => {
                // ASL A
                self.phantom_pc_read();

                self.registers.a = self.asl(self.registers.a);
            }
            0x10 => {
                // BPL rel
                self.branch(!self.registers.p_negative);
            }
            0x20 => {
                // JSR abs
                let pc_low = self.imm();

                self.phantom_stack_read();

                self.push_16(self.registers.pc);

                let pc_high = self.imm_peek();

                self.registers.pc = (pc_high as u16) << 8 | pc_low as u16;
            }
            0x26 => {
                // ROL zp
                let address = self.zpg_address();

                let old_value = self.read(address, CycleOp::Sync);

                self.write(address, old_value, CycleOp::Sync);

                let new_value = self.rol(old_value);

                self.write(address, new_value, CycleOp::Sync);
            }
            0x29 => {
                // AND imm
                let value = self.imm();

                self.and(value);
            }
            0x48 => {
                // PHA
                self.phantom_pc_read();

                self.push(self.registers.a);
            }
            0x49 => {
                // EOR imm
                let value = self.imm();

                self.xor(value);
            }
            0x4a => {
                // LSR A
                self.phantom_pc_read();

                self.registers.a = self.lsr(self.registers.a);
            }
            0x60 => {
                // RTS
                self.phantom_pc_read();

                self.phantom_stack_read();

                self.registers.pc = self.pop_16();

                self.phantom_pc_read();

                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
            0x66 => {
                // ROR zp
                let address = self.zpg_address();

                let old_value = self.read(address, CycleOp::Sync);

                self.write(address, old_value, CycleOp::Sync);

                let new_value = self.ror(old_value);

                self.write(address, new_value, CycleOp::Sync);
            }
            0x68 => {
                // PLA
                self.phantom_pc_read();

                self.phantom_stack_read();

                let value = self.pop();

                self.lda(value);
            }
            0x6a => {
                // ROR A
                self.phantom_pc_read();

                self.registers.a = self.ror(self.registers.a);
            }
            0x78 => {
                // SEI
                self.phantom_pc_read();

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
                self.phantom_pc_read();

                self.lda(self.registers.x);
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
            0x90 => {
                // BCC rel
                self.branch(!self.registers.p_carry);
            }
            0x91 => {
                // STA (zp),Y
                let address = self.ind_y_address();

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0x95 => {
                // STA zp,X
                let address = self.zpg_x_address();

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0x9a => {
                // TXS
                self.phantom_pc_read();

                self.registers.s = self.registers.x;
            }
            0x9d => {
                // STA abs,X
                let address = self.abs_offset_address(self.registers.x);

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0xa0 => {
                // LDY imm
                let value = self.imm();

                self.ldy(value);
            }
            0xa2 => {
                // LDX imm
                let value = self.imm();

                self.ldx(value);
            }
            0xa5 => {
                // LDA zp
                let value = self.zpg_address_value();

                self.lda(value);
            }
            0xa8 => {
                // TAY
                self.phantom_pc_read();

                self.ldy(self.registers.a);
            }
            0xa9 => {
                // LDA imm
                let value = self.imm();

                self.lda(value);
            }
            0xaa => {
                // TXA
                self.phantom_pc_read();

                self.ldx(self.registers.a);
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
            0xb9 => {
                // LDA abs,Y
                let value = self.abs_offset_address_value(self.registers.y);

                self.lda(value);
            }
            0xca => {
                // DEX
                self.phantom_pc_read();

                self.registers.x = self.dec(self.registers.x);
            }
            0xc5 => {
                // CMP zp
                let value = self.zpg_address_value();

                self.cmp(value);
            }
            0xc8 => {
                // INY
                self.phantom_pc_read();

                self.registers.y = self.inc(self.registers.y);
            }
            0xd0 => {
                // BNE rel
                self.branch(!self.registers.p_zero);
            }
            0xd8 => {
                // CLD
                self.phantom_pc_read();

                self.registers.p_decimal_mode = false;
            }
            0xe0 => {
                // CPX imm
                let value = self.imm();

                self.cpx(value);
            }
            0xe6 => {
                // INC zp
                let address = self.zpg_address();

                let old_value = self.read(address, CycleOp::Sync);

                self.write(address, old_value, CycleOp::Sync);

                let new_value = self.inc(old_value);

                self.write(address, new_value, CycleOp::Sync);
            }
            0xe8 => {
                // INX
                self.phantom_pc_read();

                self.registers.x = self.inc(self.registers.x);
            }
            0xee => {
                // INC abs
                let address = self.abs_address();

                let old_value = self.read(address, CycleOp::Sync);

                self.write(address, old_value, CycleOp::Sync);

                let new_value = self.inc(old_value);

                self.write(address, new_value, CycleOp::Sync);
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

    fn phantom_pc_read(&mut self) {
        self.phantom_read(self.registers.pc);
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

    fn imm_peek(&mut self) -> u8 {
        self.read(self.registers.pc, CycleOp::None)
    }

    fn imm(&mut self) -> u8 {
        let value = self.read(self.registers.pc, CycleOp::None);
        self.inc_pc();

        value
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

    fn phantom_stack_read(&mut self) {
        self.phantom_read(0x100 + (self.registers.s as u16));
    }

    fn push_16(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push((value & 0xff) as u8);
    }

    fn pop_16(&mut self) -> u16 {
        let low = self.pop();
        let high = self.pop();

        ((high as u16) << 8) | (low as u16)
    }

    fn abs_address(&mut self) -> u16 {
        let low = self.imm();
        let high = self.imm();

        ((high as u16) << 8) | (low as u16)
    }

    fn abs_address_value(&mut self) -> u8 {
        let address = self.abs_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn abs_offset_address(&mut self, offset: u8) -> u16 {
        let (address, carry_result) = address_offset(self.abs_address(), offset);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        } else {
            self.phantom_read(address);
        }

        address
    }

    fn abs_offset_address_value(&mut self, offset: u8) -> u8 {
        let (address, carry_result) = address_offset(self.abs_address(), offset);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        }

        self.read(address, CycleOp::Sync)
    }

    fn zpg_address(&mut self) -> u16 {
        let address = self.imm();

        address as u16
    }

    fn zpg_address_value(&mut self) -> u8 {
        let address = self.zpg_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn zpg_x_address(&mut self) -> u16 {
        let base_address = self.imm();

        self.phantom_read(base_address as u16);

        base_address.wrapping_add(self.registers.x) as u16
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
            self.phantom_pc_read();

            self.inc_pc();

            return;
        }

        let rel_address = self.imm();

        self.phantom_pc_read();

        let new_pc_low = (self.registers.pc & 0x00ff) + rel_address as u16;

        self.registers.pc = self.registers.pc & 0xff00 | new_pc_low & 0xff;

        let pc_high_adjustment =
            (new_pc_low & 0x100).wrapping_sub((rel_address as u16 & 0x80) << 1);

        if pc_high_adjustment != 0 {
            self.phantom_pc_read();

            self.registers.pc = self.registers.pc.wrapping_add(pc_high_adjustment);
        }
    }

    fn cmp(&mut self, value: u8) {
        self.registers.p_carry = self.registers.a >= value;
        self.registers.p_zero = self.registers.a == value;
        self.registers.p_negative = self.registers.a.wrapping_sub(value) & 0x80 != 0;
    }

    fn cpx(&mut self, value: u8) {
        self.registers.p_carry = self.registers.x >= value;
        self.registers.p_zero = self.registers.x == value;
        self.registers.p_negative = self.registers.x.wrapping_sub(value) & 0x80 != 0;
    }

    fn asl(&mut self, old_value: u8) -> u8 {
        let new_value = old_value << 1;

        self.registers.p_carry = (old_value & 0x80) != 0;

        self.set_p_zero_negative(new_value);

        new_value
    }

    fn lsr(&mut self, old_value: u8) -> u8 {
        let new_value = old_value >> 1;

        self.registers.p_carry = (old_value & 0x01) != 0;

        self.registers.p_zero = new_value == 0;
        self.registers.p_negative = false;

        new_value
    }

    fn rol(&mut self, old_value: u8) -> u8 {
        let new_value = (old_value << 1) + self.registers.p_carry as u8;

        self.registers.p_carry = (old_value & 0x80) != 0;

        self.set_p_zero_negative(new_value);

        new_value
    }

    fn ror(&mut self, old_value: u8) -> u8 {
        let new_value = (old_value >> 1) + (self.registers.p_carry as u8) * 0x80;

        self.set_p_zero_negative(new_value);

        self.registers.p_carry = (old_value & 0x01) != 0;

        new_value
    }

    fn inc(&mut self, old_val: u8) -> u8 {
        let new_value = old_val.wrapping_add(1);

        self.set_p_zero_negative(new_value);

        new_value
    }

    fn dec(&mut self, old_val: u8) -> u8 {
        let new_value = old_val.wrapping_sub(1);

        self.set_p_zero_negative(new_value);

        new_value
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

    fn xor(&mut self, operand: u8) {
        self.registers.a ^= operand;

        self.set_p_zero_negative(self.registers.a);
    }
}

fn address_offset(base_address: u16, offset: u8) -> (u16, CarryResult) {
    let address = base_address.wrapping_add(offset as u16);

    let carried = address & 0xff00 != base_address & 0xff00;

    if carried {
        let intermediate = (base_address & 0xff00) | (address & 0x00ff);
        (address, CarryResult::Carried { intermediate })
    } else {
        (address, CarryResult::NoCarry)
    }
}

enum CarryResult {
    Carried { intermediate: u16 },
    NoCarry,
}
