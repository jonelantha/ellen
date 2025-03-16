use crate::cpu::{registers::ProcessorFlags, util::*};

use super::unary_ops::shift_right;

pub type AccumulatorBinaryOpFn = fn(&mut ProcessorFlags, &mut u8, u8);

pub fn and(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    *accumulator &= operand;

    flags.update_zero_negative(*accumulator);
}

pub fn and_negative_carry(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    and(flags, accumulator, operand);

    flags.carry = flags.negative;
}

pub fn or(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    *accumulator |= operand;

    flags.update_zero_negative(*accumulator);
}

pub fn xor(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    *accumulator ^= operand;

    flags.update_zero_negative(*accumulator);
}

pub fn bit_test(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    flags.update_zero(*accumulator & operand);
    flags.overflow = operand & 0x40 != 0;
    flags.update_negative(operand);
}

pub fn and_shift_right(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    *accumulator = shift_right(flags, *accumulator & operand)
}

pub fn add_with_carry(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    if flags.decimal_mode {
        add_with_carry_bcd(flags, accumulator, operand)
    } else {
        add_with_carry_non_bcd(flags, accumulator, operand)
    }
}

fn add_with_carry_non_bcd(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    let carry = flags.carry as u8;

    let (result, operand_overflow) = accumulator.overflowing_add(operand);
    let (result, carry_overflow) = result.overflowing_add(carry);

    flags.carry = operand_overflow || carry_overflow;

    flags.update_zero_negative(result);
    flags.overflow = add_with_carry_overflow(*accumulator, result, operand);

    *accumulator = result;
}

fn add_with_carry_bcd(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    let carry_in = flags.carry as u8;

    // calculate normally for zero flag

    let result = accumulator.wrapping_add(operand);
    let result = result.wrapping_add(carry_in);

    flags.update_zero(result);

    // bcd calculation

    let low_nibble = to_low_nibble(*accumulator) + to_low_nibble(operand) + carry_in;

    let (low_nibble, low_carry_out) = wrap_nibble_up(low_nibble);

    let high_nibble = to_high_nibble(*accumulator) + to_high_nibble(operand) + low_carry_out;

    // N and V are determined before high nibble is adjusted
    let result_so_far = from_nibbles(high_nibble, low_nibble);
    flags.overflow = add_with_carry_overflow(*accumulator, result_so_far, operand);
    flags.update_negative(result_so_far);

    let (high_nibble, high_carry_out) = wrap_nibble_up(high_nibble);

    flags.carry = high_carry_out == 1;

    *accumulator = from_nibbles(high_nibble, low_nibble);
}

pub fn subtract_with_carry(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    if flags.decimal_mode {
        subtract_with_carry_bcd(flags, accumulator, operand)
    } else {
        add_with_carry_non_bcd(flags, accumulator, !operand)
    }
}

fn subtract_with_carry_bcd(flags: &mut ProcessorFlags, accumulator: &mut u8, operand: u8) {
    let borrow_in = 1 - flags.carry as u8;

    // calculate normally for flags

    let result = accumulator.wrapping_sub(operand);
    let result = result.wrapping_sub(borrow_in);

    flags.update_zero_negative(result);
    flags.overflow = subtract_with_carry_overflow(*accumulator, result, operand);

    // then calculate for BCD

    let low_nibble = to_low_nibble(*accumulator)
        .wrapping_sub(to_low_nibble(operand))
        .wrapping_sub(borrow_in);

    let (low_nibble, low_borrow_out) = wrap_nibble_down(low_nibble);

    let high_nibble = to_high_nibble(*accumulator)
        .wrapping_sub(to_high_nibble(operand))
        .wrapping_sub(low_borrow_out);

    let (high_nibble, high_borrow_out) = wrap_nibble_down(high_nibble);

    flags.carry = high_borrow_out == 0;

    *accumulator = from_nibbles(high_nibble, low_nibble);
}

fn add_with_carry_overflow(accumulator: u8, result: u8, operand: u8) -> bool {
    is_negative((accumulator ^ result) & (accumulator ^ !operand))
}

fn subtract_with_carry_overflow(accumulator: u8, result: u8, operand: u8) -> bool {
    add_with_carry_overflow(accumulator, result, !operand)
}
