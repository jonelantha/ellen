use crate::cpu::registers::ProcessorFlags;

pub type UnaryOp = fn(&mut ProcessorFlags, u8) -> u8;

pub fn shift_left(flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = old_value << 1;

    flags.carry = (old_value & 0x80) != 0;

    flags.update_zero_negative(new_value);

    new_value
}

pub fn shift_right(flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = old_value >> 1;

    flags.carry = (old_value & 0x01) != 0;

    flags.update_zero(new_value);
    flags.negative = false;

    new_value
}

pub fn rotate_left(flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = (old_value << 1) | flags.carry as u8;

    flags.carry = (old_value & 0x80) != 0;

    flags.update_zero_negative(new_value);

    new_value
}

pub fn rotate_right(flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = (old_value >> 1) | ((flags.carry as u8) * 0x80);

    flags.update_zero_negative(new_value);

    flags.carry = (old_value & 0x01) != 0;

    new_value
}

pub fn increment(flags: &mut ProcessorFlags, old_val: u8) -> u8 {
    let new_value = old_val.wrapping_add(1);

    flags.update_zero_negative(new_value);

    new_value
}

pub fn decrement(flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = old_value.wrapping_sub(1);

    flags.update_zero_negative(new_value);

    new_value
}
