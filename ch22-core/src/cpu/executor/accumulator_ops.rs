use crate::cpu::registers::Registers;

use super::byte_ops::shift_right;

pub fn and(registers: &mut Registers, operand: u8) {
    registers.a &= operand;

    registers.p.update_zero_negative(registers.a);
}

pub fn and_negative_carry(registers: &mut Registers, operand: u8) {
    and(registers, operand);

    registers.p.carry = registers.p.negative;
}

pub fn or(registers: &mut Registers, operand: u8) {
    registers.a |= operand;

    registers.p.update_zero_negative(registers.a);
}

pub fn xor(registers: &mut Registers, operand: u8) {
    registers.a ^= operand;

    registers.p.update_zero_negative(registers.a);
}

pub fn bit_test(registers: &mut Registers, operand: u8) {
    registers.p.update_zero(registers.a & operand);
    registers.p.overflow = operand & 0x40 != 0;
    registers.p.update_negative(operand);
}

pub fn and_shift_right(registers: &mut Registers, operand: u8) {
    registers.a = shift_right(&mut registers.p, registers.a & operand);
}

pub fn add_with_carry(registers: &mut Registers, operand: u8) {
    if registers.p.decimal_mode {
        add_with_carry_bcd(registers, operand);
    } else {
        add_with_carry_non_bcd(registers, operand);
    }
}

fn add_with_carry_non_bcd(registers: &mut Registers, operand: u8) {
    let carry = registers.p.carry as u8;

    let (result, operand_overflow) = registers.a.overflowing_add(operand);
    let (result, carry_overflow) = result.overflowing_add(carry);

    registers.p.carry = operand_overflow || carry_overflow;

    registers.p.update_zero_negative(result);
    set_add_with_carry_overflow(registers, result, operand);

    registers.a = result;
}

fn add_with_carry_bcd(registers: &mut Registers, operand: u8) {
    let carry_in = registers.p.carry as u8;

    // calculate normally for zero flag

    let result = registers.a.wrapping_add(operand);
    let result = result.wrapping_add(carry_in);

    registers.p.update_zero(result);

    // bcd calculation

    let low_nibble = to_low_nibble(registers.a) + to_low_nibble(operand) + carry_in;

    let (low_nibble, low_carry_out) = wrap_nibble_up(low_nibble);

    let high_nibble = to_high_nibble(registers.a) + to_high_nibble(operand) + low_carry_out;

    // N and V are determined before high nibble is adjusted
    let result_so_far = from_nibbles(high_nibble, low_nibble);
    set_add_with_carry_overflow(registers, result_so_far, operand);
    registers.p.update_negative(result_so_far);

    let (high_nibble, high_carry_out) = wrap_nibble_up(high_nibble);

    registers.p.carry = high_carry_out == 1;

    registers.a = from_nibbles(high_nibble, low_nibble);
}

pub fn subtract_with_carry(registers: &mut Registers, operand: u8) {
    if registers.p.decimal_mode {
        subtract_with_carry_bcd(registers, operand);
    } else {
        add_with_carry_non_bcd(registers, !operand);
    }
}

fn subtract_with_carry_bcd(registers: &mut Registers, operand: u8) {
    let borrow_in = 1 - registers.p.carry as u8;

    // calculate normally for flags

    let result = registers.a.wrapping_sub(operand);
    let result = result.wrapping_sub(borrow_in);

    registers.p.update_zero_negative(result);
    set_subtract_with_carry_overflow(registers, result, operand);

    // then calculate for BCD

    let low_nibble = to_low_nibble(registers.a)
        .wrapping_sub(to_low_nibble(operand))
        .wrapping_sub(borrow_in);

    let (low_nibble, low_borrow_out) = wrap_nibble_down(low_nibble);

    let high_nibble = to_high_nibble(registers.a)
        .wrapping_sub(to_high_nibble(operand))
        .wrapping_sub(low_borrow_out);

    let (high_nibble, high_borrow_out) = wrap_nibble_down(high_nibble);

    registers.p.carry = high_borrow_out == 0;

    registers.a = from_nibbles(high_nibble, low_nibble);
}

fn set_add_with_carry_overflow(registers: &mut Registers, result: u8, operand: u8) {
    registers.p.overflow = is_negative((registers.a ^ result) & (registers.a ^ !operand));
}

fn set_subtract_with_carry_overflow(registers: &mut Registers, result: u8, operand: u8) {
    set_add_with_carry_overflow(registers, result, !operand);
}

// helpers

fn is_negative(value: u8) -> bool {
    value & 0x80 != 0
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
