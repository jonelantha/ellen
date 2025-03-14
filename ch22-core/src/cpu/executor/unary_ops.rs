use crate::cpu::registers::ProcessorFlags;

pub fn shift_left(processor_flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = old_value << 1;

    processor_flags.carry = (old_value & 0x80) != 0;

    processor_flags.update_zero_negative(new_value);

    new_value
}

pub fn shift_right(processor_flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = old_value >> 1;

    processor_flags.carry = (old_value & 0x01) != 0;

    processor_flags.update_zero(new_value);
    processor_flags.negative = false;

    new_value
}

pub fn rotate_left(processor_flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = (old_value << 1) | processor_flags.carry as u8;

    processor_flags.carry = (old_value & 0x80) != 0;

    processor_flags.update_zero_negative(new_value);

    new_value
}

pub fn rotate_right(processor_flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = (old_value >> 1) | ((processor_flags.carry as u8) * 0x80);

    processor_flags.update_zero_negative(new_value);

    processor_flags.carry = (old_value & 0x01) != 0;

    new_value
}

pub fn increment(processor_flags: &mut ProcessorFlags, old_val: u8) -> u8 {
    let new_value = old_val.wrapping_add(1);

    processor_flags.update_zero_negative(new_value);

    new_value
}

pub fn decrement(processor_flags: &mut ProcessorFlags, old_value: u8) -> u8 {
    let new_value = old_value.wrapping_sub(1);

    processor_flags.update_zero_negative(new_value);

    new_value
}
