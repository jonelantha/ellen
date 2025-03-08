#[derive(Default)]
pub struct Registers {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: StatusRegister,
}

impl Registers {
    pub fn get(&self, register_type: RegisterType) -> u8 {
        match register_type {
            RegisterType::Stack => self.s,
            RegisterType::Accumulator => self.a,
            RegisterType::X => self.x,
            RegisterType::Y => self.y,
        }
    }

    pub fn set(&mut self, register_type: RegisterType, value: u8) {
        match register_type {
            RegisterType::Stack => {
                self.s = value;
            }
            RegisterType::Accumulator => {
                self.a = value;
            }
            RegisterType::X => {
                self.x = value;
            }
            RegisterType::Y => {
                self.y = value;
            }
        }
    }

    pub fn set_with_flags(&mut self, register_type: RegisterType, value: u8) {
        self.set(register_type, value);

        self.set_p_zero_negative(value);
    }

    pub fn compare(&mut self, value: u8, register: u8) {
        self.p.carry = register >= value;
        self.p.zero = register == value;
        self.set_p_negative(register.wrapping_sub(value));
    }

    pub fn shift_left(&mut self, old_value: u8) -> u8 {
        let new_value = old_value << 1;

        self.p.carry = (old_value & 0x80) != 0;

        self.set_p_zero_negative(new_value);

        new_value
    }

    pub fn shift_right(&mut self, old_value: u8) -> u8 {
        let new_value = old_value >> 1;

        self.p.carry = (old_value & 0x01) != 0;

        self.set_p_zero(new_value);
        self.p.negative = false;

        new_value
    }

    pub fn rotate_left(&mut self, old_value: u8) -> u8 {
        let new_value = (old_value << 1) | self.p.carry as u8;

        self.p.carry = (old_value & 0x80) != 0;

        self.set_p_zero_negative(new_value);

        new_value
    }

    pub fn rotate_right(&mut self, old_value: u8) -> u8 {
        let new_value = (old_value >> 1) | (self.p.carry as u8) * 0x80;

        self.set_p_zero_negative(new_value);

        self.p.carry = (old_value & 0x01) != 0;

        new_value
    }

    pub fn accumulator_shift_left_or(&mut self, old_value: u8) -> u8 {
        let new_value = old_value << 1;

        self.p.carry = (old_value & 0x80) != 0;

        self.a |= new_value;

        self.set_p_zero_negative(self.a);

        new_value
    }

    pub fn increment(&mut self, old_val: u8) -> u8 {
        let new_value = old_val.wrapping_add(1);

        self.set_p_zero_negative(new_value);

        new_value
    }

    pub fn decrement(&mut self, old_value: u8) -> u8 {
        let new_value = old_value.wrapping_sub(1);

        self.set_p_zero_negative(new_value);

        new_value
    }

    pub fn accumulator_and(&mut self, operand: u8) {
        self.a &= operand;

        self.set_p_zero_negative(self.a);
    }

    pub fn and_negative_carry(&mut self, operand: u8) {
        self.accumulator_and(operand);

        self.p.carry = self.p.negative;
    }

    pub fn accumulator_or(&mut self, operand: u8) {
        self.a |= operand;

        self.set_p_zero_negative(self.a);
    }

    pub fn accumulator_and_shift_right(&mut self, operand: u8) {
        self.a = self.shift_right(self.a & operand);
    }

    pub fn accumulator_xor(&mut self, operand: u8) {
        self.a ^= operand;

        self.set_p_zero_negative(self.a);
    }

    pub fn accumulator_bit_test(&mut self, operand: u8) {
        self.set_p_zero(self.a & operand);
        self.p.overflow = operand & 0x40 != 0;
        self.set_p_negative(operand);
    }

    pub fn add_with_carry(&mut self, operand: u8) {
        if self.p.decimal_mode {
            self.add_with_carry_bcd(operand);
        } else {
            self.add_with_carry_non_bcd(operand);
        }
    }

    fn add_with_carry_non_bcd(&mut self, operand: u8) {
        let carry = self.p.carry as u8;

        let (result, operand_overflow) = self.a.overflowing_add(operand);
        let (result, carry_overflow) = result.overflowing_add(carry);

        self.p.carry = operand_overflow || carry_overflow;

        self.set_p_zero_negative(result);
        self.set_overflow_add_with_carry(result, operand);

        self.a = result;
    }

    fn add_with_carry_bcd(&mut self, operand: u8) {
        let carry_in = self.p.carry as u8;

        // calculate normally for zero flag

        let result = self.a.wrapping_add(operand);
        let result = result.wrapping_add(carry_in);

        self.set_p_zero(result);

        // bcd calculation

        let low_nibble = to_low_nibble(self.a) + to_low_nibble(operand) + carry_in;

        let (low_nibble, low_carry_out) = wrap_nibble_up(low_nibble);

        let high_nibble = to_high_nibble(self.a) + to_high_nibble(operand) + low_carry_out;

        // N and V are determined before high nibble is adjusted
        let result_so_far = from_nibbles(high_nibble, low_nibble);
        self.set_overflow_add_with_carry(result_so_far, operand);
        self.set_p_negative(result_so_far);

        let (high_nibble, high_carry_out) = wrap_nibble_up(high_nibble);

        self.p.carry = high_carry_out == 1;

        self.a = from_nibbles(high_nibble, low_nibble);
    }

    pub fn substract_with_carry(&mut self, operand: u8) {
        if self.p.decimal_mode {
            self.subtract_with_carry_bcd(operand);
        } else {
            self.add_with_carry_non_bcd(!operand);
        }
    }

    fn subtract_with_carry_bcd(&mut self, operand: u8) {
        let borrow_in = 1 - self.p.carry as u8;

        // calculate normally for flags

        let result = self.a.wrapping_sub(operand);
        let result = result.wrapping_sub(borrow_in);

        self.set_p_zero_negative(result);
        self.set_overflow_subtract_with_carry(result, operand);

        // then calculate for BCD

        let low_nibble = to_low_nibble(self.a)
            .wrapping_sub(to_low_nibble(operand))
            .wrapping_sub(borrow_in);

        let (low_nibble, low_borrow_out) = wrap_nibble_down(low_nibble);

        let high_nibble = to_high_nibble(self.a)
            .wrapping_sub(to_high_nibble(operand))
            .wrapping_sub(low_borrow_out);

        let (high_nibble, high_borrow_out) = wrap_nibble_down(high_nibble);

        self.p.carry = high_borrow_out == 0;

        self.a = from_nibbles(high_nibble, low_nibble);
    }

    fn set_overflow_add_with_carry(&mut self, result: u8, operand: u8) {
        self.p.overflow = is_negative((self.a ^ result) & (self.a ^ !operand));
    }

    fn set_overflow_subtract_with_carry(&mut self, result: u8, operand: u8) {
        self.set_overflow_add_with_carry(result, !operand);
    }

    fn set_p_negative(&mut self, value: u8) {
        self.p.negative = is_negative(value);
    }

    fn set_p_zero(&mut self, value: u8) {
        self.p.zero = value == 0;
    }

    fn set_p_zero_negative(&mut self, value: u8) {
        self.set_p_zero(value);
        self.set_p_negative(value);
    }
}

#[derive(Clone, Copy)]
pub enum RegisterType {
    Stack,
    Accumulator,
    X,
    Y,
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

#[derive(Default, PartialEq, Debug)]
pub struct StatusRegister {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal_mode: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl From<u8> for StatusRegister {
    fn from(flags: u8) -> Self {
        StatusRegister {
            carry: flags & P_CARRY_FLAG != 0,
            zero: flags & P_ZERO_FLAG != 0,
            interrupt_disable: flags & P_I_INTERRUPT_DISABLE_FLAG != 0,
            decimal_mode: flags & P_DECIMAL_MODE_FLAG != 0,
            overflow: flags & P_OVERFLOW_FLAG != 0,
            negative: flags & P_NEGATIVE_FLAG != 0,
        }
    }
}

impl From<&StatusRegister> for u8 {
    fn from(p: &StatusRegister) -> Self {
        (if p.carry { P_CARRY_FLAG } else { 0 })
            | (if p.zero { P_ZERO_FLAG } else { 0 })
            | (if p.interrupt_disable {
                P_I_INTERRUPT_DISABLE_FLAG
            } else {
                0
            })
            | (if p.decimal_mode {
                P_DECIMAL_MODE_FLAG
            } else {
                0
            })
            | P_BIT_5_FLAG
            | (if p.overflow { P_OVERFLOW_FLAG } else { 0 })
            | (if p.negative { P_NEGATIVE_FLAG } else { 0 })
    }
}

pub const P_CARRY_FLAG: u8 = 0b00000001;
pub const P_ZERO_FLAG: u8 = 0b00000010;
pub const P_I_INTERRUPT_DISABLE_FLAG: u8 = 0b00000100;
pub const P_DECIMAL_MODE_FLAG: u8 = 0b00001000;
pub const P_BREAK_FLAG: u8 = 0b00010000;
pub const P_BIT_5_FLAG: u8 = 0b00100000;
pub const P_OVERFLOW_FLAG: u8 = 0b01000000;
pub const P_NEGATIVE_FLAG: u8 = 0b10000000;
