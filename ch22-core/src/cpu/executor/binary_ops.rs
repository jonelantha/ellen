use crate::cpu::{registers::ProcessorFlags, util::*};

use super::unary_ops::shift_right;

pub type BinaryOp = fn(&mut ProcessorFlags, u8, u8) -> u8;

pub fn and(processor_flags: &mut ProcessorFlags, mut accumulator: u8, operand: u8) -> u8 {
    accumulator &= operand;

    processor_flags.update_zero_negative(accumulator);

    accumulator
}

pub fn and_negative_carry(
    processor_flags: &mut ProcessorFlags,
    mut accumulator: u8,
    operand: u8,
) -> u8 {
    accumulator = and(processor_flags, accumulator, operand);

    processor_flags.carry = processor_flags.negative;

    accumulator
}

pub fn or(processor_flags: &mut ProcessorFlags, mut accumulator: u8, operand: u8) -> u8 {
    accumulator |= operand;

    processor_flags.update_zero_negative(accumulator);

    accumulator
}

pub fn xor(processor_flags: &mut ProcessorFlags, mut accumulator: u8, operand: u8) -> u8 {
    accumulator ^= operand;

    processor_flags.update_zero_negative(accumulator);

    accumulator
}

pub fn bit_test(processor_flags: &mut ProcessorFlags, accumulator: u8, operand: u8) -> u8 {
    processor_flags.update_zero(accumulator & operand);
    processor_flags.overflow = operand & 0x40 != 0;
    processor_flags.update_negative(operand);

    accumulator
}

pub fn and_shift_right(processor_flags: &mut ProcessorFlags, accumulator: u8, operand: u8) -> u8 {
    shift_right(processor_flags, accumulator & operand)
}

pub fn add_with_carry(processor_flags: &mut ProcessorFlags, accumulator: u8, operand: u8) -> u8 {
    if processor_flags.decimal_mode {
        add_with_carry_bcd(processor_flags, accumulator, operand)
    } else {
        add_with_carry_non_bcd(processor_flags, accumulator, operand)
    }
}

fn add_with_carry_non_bcd(
    processor_flags: &mut ProcessorFlags,
    accumulator: u8,
    operand: u8,
) -> u8 {
    let carry = processor_flags.carry as u8;

    let (result, operand_overflow) = accumulator.overflowing_add(operand);
    let (result, carry_overflow) = result.overflowing_add(carry);

    processor_flags.carry = operand_overflow || carry_overflow;

    processor_flags.update_zero_negative(result);
    processor_flags.overflow = add_with_carry_overflow(accumulator, result, operand);

    result
}

fn add_with_carry_bcd(processor_flags: &mut ProcessorFlags, accumulator: u8, operand: u8) -> u8 {
    let carry_in = processor_flags.carry as u8;

    // calculate normally for zero flag

    let result = accumulator.wrapping_add(operand);
    let result = result.wrapping_add(carry_in);

    processor_flags.update_zero(result);

    // bcd calculation

    let low_nibble = to_low_nibble(accumulator) + to_low_nibble(operand) + carry_in;

    let (low_nibble, low_carry_out) = wrap_nibble_up(low_nibble);

    let high_nibble = to_high_nibble(accumulator) + to_high_nibble(operand) + low_carry_out;

    // N and V are determined before high nibble is adjusted
    let result_so_far = from_nibbles(high_nibble, low_nibble);
    processor_flags.overflow = add_with_carry_overflow(accumulator, result_so_far, operand);
    processor_flags.update_negative(result_so_far);

    let (high_nibble, high_carry_out) = wrap_nibble_up(high_nibble);

    processor_flags.carry = high_carry_out == 1;

    from_nibbles(high_nibble, low_nibble)
}

pub fn subtract_with_carry(
    processor_flags: &mut ProcessorFlags,
    accumulator: u8,
    operand: u8,
) -> u8 {
    if processor_flags.decimal_mode {
        subtract_with_carry_bcd(processor_flags, accumulator, operand)
    } else {
        add_with_carry_non_bcd(processor_flags, accumulator, !operand)
    }
}

fn subtract_with_carry_bcd(
    processor_flags: &mut ProcessorFlags,
    accumulator: u8,
    operand: u8,
) -> u8 {
    let borrow_in = 1 - processor_flags.carry as u8;

    // calculate normally for flags

    let result = accumulator.wrapping_sub(operand);
    let result = result.wrapping_sub(borrow_in);

    processor_flags.update_zero_negative(result);
    processor_flags.overflow = subtract_with_carry_overflow(accumulator, result, operand);

    // then calculate for BCD

    let low_nibble = to_low_nibble(accumulator)
        .wrapping_sub(to_low_nibble(operand))
        .wrapping_sub(borrow_in);

    let (low_nibble, low_borrow_out) = wrap_nibble_down(low_nibble);

    let high_nibble = to_high_nibble(accumulator)
        .wrapping_sub(to_high_nibble(operand))
        .wrapping_sub(low_borrow_out);

    let (high_nibble, high_borrow_out) = wrap_nibble_down(high_nibble);

    processor_flags.carry = high_borrow_out == 0;

    from_nibbles(high_nibble, low_nibble)
}

fn add_with_carry_overflow(accumulator: u8, result: u8, operand: u8) -> bool {
    is_negative((accumulator ^ result) & (accumulator ^ !operand))
}

fn subtract_with_carry_overflow(accumulator: u8, result: u8, operand: u8) -> bool {
    add_with_carry_overflow(accumulator, result, !operand)
}
