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

    pub fn execute(&mut self, ignore_69: bool) -> Option<u8> {
        let opcode = self.imm();

        if opcode == 0x69 && ignore_69 {
            return Some(opcode);
        }

        match opcode {
            0x08 => {
                // PHP
                self.phantom_pc_read();

                self.push(self.registers.get_p() | P_BREAK_FLAG);
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
            0x18 => {
                // CLC
                self.phantom_pc_read();

                self.registers.p_carry = false;
            }
            0x1d => {
                // ORA abs,X
                let value = self.abs_offset_address_value(self.registers.x);

                self.or(value);
            }
            0x20 => {
                // JSR abs
                let pc_low = self.imm();

                self.phantom_stack_read();

                self.push_16(self.registers.pc);

                let pc_high = self.imm_peek();

                self.registers.pc = u16::from_le_bytes([pc_low, pc_high]);
            }
            0x24 => {
                // BIT zp
                let value = self.zpg_address_value();

                self.bit(value);
            }
            0x26 => {
                // ROL zp
                let address = self.zpg_address();

                let old_value = self.read(address, CycleOp::Sync);

                self.write(address, old_value, CycleOp::Sync);

                let new_value = self.rol(old_value);

                self.write(address, new_value, CycleOp::Sync);
            }
            0x28 => {
                // PLP
                self.phantom_pc_read();

                self.phantom_stack_read();

                let value = self.pop();

                self.registers.set_p(value);
            }
            0x29 => {
                // AND imm
                let value = self.imm();

                self.and(value);
            }
            0x2c => {
                // BIT abs
                let value = self.abs_address_value();

                self.bit(value);
            }
            0x30 => {
                // BMI rel
                self.branch(self.registers.p_negative);
            }
            0x38 => {
                // SEC
                self.phantom_pc_read();

                self.registers.p_carry = true;
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
            0x4c => {
                // JMP abs
                self.registers.pc = self.abs_address();
            }
            0x4e => {
                // LSR abs
                let address = self.abs_address();

                let old_value = self.read(address, CycleOp::Sync);

                self.write(address, old_value, CycleOp::Sync);

                let new_value = self.lsr(old_value);

                self.write(address, new_value, CycleOp::Sync);
            }
            0x50 => {
                // BVC rel
                self.branch(!self.registers.p_overflow)
            }
            0x58 => {
                // CLI
                self.phantom_pc_read();

                self.registers.p_interrupt_disable = false;
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
            0x69 => {
                // ADC imm
                let value = self.imm();

                self.adc(value);
            }
            0x78 => {
                // SEI
                self.phantom_pc_read();

                self.registers.p_interrupt_disable = true;
            }
            0x7d => {
                // ADC abs,X
                let value = self.abs_offset_address_value(self.registers.x);

                self.adc(value);
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
            0x88 => {
                // DEY
                self.phantom_pc_read();

                self.registers.y = self.dec(self.registers.y);
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
            0x99 => {
                // STA abs,Y
                let address = self.abs_offset_address(self.registers.y);

                self.write(address, self.registers.a, CycleOp::CheckInterrupt);
            }
            0x98 => {
                // TYA
                self.phantom_pc_read();

                self.lda(self.registers.y);
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
            0xa4 => {
                // LDY zp
                let value = self.zpg_address_value();

                self.ldy(value);
            }
            0xa5 => {
                // LDA zp
                let value = self.zpg_address_value();

                self.lda(value);
            }
            0xa6 => {
                // LDX zp
                let value = self.zpg_address_value();

                self.ldx(value);
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
            0xac => {
                // LDY abs
                let value = self.abs_address_value();

                self.ldy(value);
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
            0xb1 => {
                // LDA (zp),Y
                let value = self.ind_y_address_value();

                self.lda(value);
            }
            0xb9 => {
                // LDA abs,Y
                let value = self.abs_offset_address_value(self.registers.y);

                self.lda(value);
            }
            0xc0 => {
                // CPY imm
                let value = self.imm();

                self.cpy(value);
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
            0xc9 => {
                // CMP abs
                let value = self.imm();

                self.cmp(value);
            }
            0xca => {
                // DEX
                self.phantom_pc_read();

                self.registers.x = self.dec(self.registers.x);
            }
            0xcd => {
                // CMP abs
                let value = self.abs_address_value();

                self.cmp(value);
            }
            0xd0 => {
                // BNE rel
                self.branch(!self.registers.p_zero);
            }
            0xd1 => {
                // CMP (zp),Y
                let value = self.ind_y_address_value();

                self.cmp(value);
            }
            0xd8 => {
                // CLD
                self.phantom_pc_read();

                self.registers.p_decimal_mode = false;
            }
            0xdd => {
                // CMP abs,X
                let value = self.abs_offset_address_value(self.registers.x);

                self.cmp(value);
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
            0xe9 => {
                // SBC imm
                let value = self.imm();

                self.sbc(value)
            }
            0xec => {
                // CPX abs
                let value = self.abs_address_value();

                self.cpx(value);
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
        let [high, low] = value.to_le_bytes();
        self.push(low);
        self.push(high);
    }

    fn pop_16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    fn abs_address(&mut self) -> u16 {
        u16::from_le_bytes([self.imm(), self.imm()])
    }

    fn abs_address_value(&mut self) -> u8 {
        let address = self.abs_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn abs_offset_address(&mut self, offset: u8) -> u16 {
        let (address, carry_result) = address_offset_unsigned(self.abs_address(), offset);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        } else {
            self.phantom_read(address);
        }

        address
    }

    fn abs_offset_address_value(&mut self, offset: u8) -> u8 {
        let (address, carry_result) = address_offset_unsigned(self.abs_address(), offset);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        }

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn zpg_address(&mut self) -> u16 {
        let address = self.imm();

        address as u16
    }

    fn zpg_address_value(&mut self) -> u8 {
        let address = self.zpg_address();

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn zpg_address_value_16(&mut self) -> u16 {
        let zpg_address = self.zpg_address();

        u16::from_le_bytes([
            self.read(zpg_address, CycleOp::None),
            self.read((zpg_address + 1) & 0xff, CycleOp::None),
        ])
    }

    fn zpg_x_address(&mut self) -> u16 {
        let base_address = self.imm();

        self.phantom_read(base_address as u16);

        base_address.wrapping_add(self.registers.x) as u16
    }

    fn ind_y_address(&mut self) -> u16 {
        let (address, carry_result) =
            address_offset_unsigned(self.zpg_address_value_16(), self.registers.y);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        } else {
            self.phantom_read(address);
        }

        address
    }

    fn ind_y_address_value(&mut self) -> u8 {
        let (address, carry_result) =
            address_offset_unsigned(self.zpg_address_value_16(), self.registers.y);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        }

        self.read(address, CycleOp::CheckInterrupt)
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.phantom_pc_read();

            self.inc_pc();

            return;
        }

        let rel_address = self.imm() as i8;

        self.phantom_pc_read();

        let (address, carry_result) = address_offset_signed(self.registers.pc, rel_address);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        }

        self.registers.pc = address;
    }

    fn cmp(&mut self, value: u8) {
        self.compare(value, self.registers.a);
    }

    fn cpx(&mut self, value: u8) {
        self.compare(value, self.registers.x);
    }

    fn cpy(&mut self, value: u8) {
        self.compare(value, self.registers.y);
    }

    fn compare(&mut self, value: u8, register: u8) {
        self.registers.p_carry = register >= value;
        self.registers.p_zero = register == value;
        self.set_p_negative(register.wrapping_sub(value));
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

        self.set_p_zero(new_value);
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

    fn bit(&mut self, operand: u8) {
        self.set_p_zero(self.registers.a & operand);
        self.registers.p_overflow = operand & 0x40 != 0;
        self.set_p_negative(operand);
    }

    fn adc(&mut self, operand: u8) {
        if self.registers.p_decimal_mode {
            self.adc_bcd(operand);
        } else {
            self.adc_bin(operand);
        }
    }

    fn adc_bin(&mut self, operand: u8) {
        let carry = self.registers.p_carry as u8;

        let (result, operand_overflow) = self.registers.a.overflowing_add(operand);
        let (result, carry_overflow) = result.overflowing_add(carry);

        self.registers.p_carry = operand_overflow || carry_overflow;

        self.set_p_zero_negative(result);
        self.set_overflow_adc(result, operand);

        self.registers.a = result;
    }

    fn adc_bcd(&mut self, operand: u8) {
        let carry_in = self.registers.p_carry as u8;

        // calculate normally for zero flag

        let result = self.registers.a.wrapping_add(operand);
        let result = result.wrapping_add(carry_in);

        self.set_p_zero(result);

        // bcd calculation

        let low_nibble = to_low_nibble(self.registers.a) + to_low_nibble(operand) + carry_in;

        let (low_nibble, low_carry_out) = wrap_nibble_up(low_nibble);

        let high_nibble =
            to_high_nibble(self.registers.a) + to_high_nibble(operand) + low_carry_out;

        // N and V are determined before high nibble is adjusted
        let result_so_far = from_nibbles(high_nibble, low_nibble);
        self.set_overflow_adc(result_so_far, operand);
        self.set_p_negative(result_so_far);

        let (high_nibble, high_carry_out) = wrap_nibble_up(high_nibble);

        self.registers.p_carry = high_carry_out == 1;

        self.registers.a = from_nibbles(high_nibble, low_nibble);
    }

    fn sbc(&mut self, operand: u8) {
        if self.registers.p_decimal_mode {
            self.sbc_bcd(operand);
        } else {
            self.adc_bin(!operand);
        }
    }

    fn sbc_bcd(&mut self, operand: u8) {
        let borrow_in = 1 - self.registers.p_carry as u8;

        // calculate normally for flags

        let result = self.registers.a.wrapping_sub(operand);
        let result = result.wrapping_sub(borrow_in);

        self.set_p_zero_negative(result);
        self.set_overflow_sbc(result, operand);

        // then calculate for BCD

        let low_nibble = to_low_nibble(self.registers.a)
            .wrapping_sub(to_low_nibble(operand))
            .wrapping_sub(borrow_in);

        let (low_nibble, low_borrow_out) = wrap_nibble_down(low_nibble);

        let high_nibble = to_high_nibble(self.registers.a)
            .wrapping_sub(to_high_nibble(operand))
            .wrapping_sub(low_borrow_out);

        let (high_nibble, high_borrow_out) = wrap_nibble_down(high_nibble);

        self.registers.p_carry = high_borrow_out == 0;

        self.registers.a = from_nibbles(high_nibble, low_nibble);
    }

    // flag helpers

    fn set_overflow_adc(&mut self, result: u8, operand: u8) {
        self.registers.p_overflow =
            is_negative((self.registers.a ^ result) & (self.registers.a ^ !operand));
    }

    fn set_overflow_sbc(&mut self, result: u8, operand: u8) {
        self.set_overflow_adc(result, !operand);
    }

    fn set_p_negative(&mut self, value: u8) {
        self.registers.p_negative = is_negative(value);
    }

    fn set_p_zero(&mut self, value: u8) {
        self.registers.p_zero = value == 0;
    }

    fn set_p_zero_negative(&mut self, value: u8) {
        self.set_p_zero(value);
        self.set_p_negative(value);
    }
}

fn is_negative(value: u8) -> bool {
    value & 0x80 != 0
}

fn address_offset(base_address: u16, offset: i16) -> (u16, CarryResult) {
    let address = base_address.wrapping_add(offset as u16);

    let carried = address & 0xff00 != base_address & 0xff00;

    if carried {
        let intermediate = (base_address & 0xff00) | (address & 0x00ff);
        (address, CarryResult::Carried { intermediate })
    } else {
        (address, CarryResult::NoCarry)
    }
}

fn address_offset_unsigned(base_address: u16, offset: u8) -> (u16, CarryResult) {
    address_offset(base_address, offset as i16)
}

fn address_offset_signed(base_address: u16, offset: i8) -> (u16, CarryResult) {
    address_offset(base_address, offset as i16)
}

enum CarryResult {
    Carried { intermediate: u16 },
    NoCarry,
}

// nibble helpers

fn wrap_nibble_up(nibble: u8) -> (u8, u8) {
    if nibble > 0x09 {
        (nibble + 0x06, 1)
    } else {
        (nibble, 0)
    }
}

fn wrap_nibble_down(nibble: u8) -> (u8, u8) {
    if nibble & 0x10 != 0 {
        (nibble - 0x06, 1)
    } else {
        (nibble, 0)
    }
}

fn from_nibbles(high: u8, low: u8) -> u8 {
    (high << 4) | (low & 0x0f)
}

fn to_high_nibble(value: u8) -> u8 {
    value >> 4
}

fn to_low_nibble(value: u8) -> u8 {
    value & 0x0f
}
