use crate::cpu::registers::StatusRegister;

pub fn shift_left(p: &mut StatusRegister, old_value: u8) -> u8 {
    let new_value = old_value << 1;

    p.carry = (old_value & 0x80) != 0;

    p.update_zero_negative(new_value);

    new_value
}

pub fn shift_right(p: &mut StatusRegister, old_value: u8) -> u8 {
    let new_value = old_value >> 1;

    p.carry = (old_value & 0x01) != 0;

    p.update_zero(new_value);
    p.negative = false;

    new_value
}

pub fn rotate_left(p: &mut StatusRegister, old_value: u8) -> u8 {
    let new_value = (old_value << 1) | p.carry as u8;

    p.carry = (old_value & 0x80) != 0;

    p.update_zero_negative(new_value);

    new_value
}

pub fn rotate_right(p: &mut StatusRegister, old_value: u8) -> u8 {
    let new_value = (old_value >> 1) | (p.carry as u8) * 0x80;

    p.update_zero_negative(new_value);

    p.carry = (old_value & 0x01) != 0;

    new_value
}

pub fn increment(p: &mut StatusRegister, old_val: u8) -> u8 {
    let new_value = old_val.wrapping_add(1);

    p.update_zero_negative(new_value);

    new_value
}

pub fn decrement(p: &mut StatusRegister, old_value: u8) -> u8 {
    let new_value = old_value.wrapping_sub(1);

    p.update_zero_negative(new_value);

    new_value
}
