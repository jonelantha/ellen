use super::util::*;

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

        self.p.update_zero_negative(value);
    }
}

#[derive(Clone, Copy)]
pub enum RegisterType {
    Stack,
    Accumulator,
    X,
    Y,
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

impl StatusRegister {
    pub fn update_zero_negative(self: &mut Self, value: u8) {
        self.update_zero(value);
        self.update_negative(value);
    }

    pub fn update_zero(self: &mut Self, value: u8) {
        self.zero = value == 0;
    }

    pub fn update_negative(self: &mut Self, value: u8) {
        self.negative = is_negative(value);
    }
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
