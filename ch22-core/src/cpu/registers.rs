pub struct Registers {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p_carry: bool,
    pub p_zero: bool,
    pub p_interrupt_disable: bool,
    pub p_decimal_mode: bool,
    pub p_overflow: bool,
    pub p_negative: bool,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            pc: 0,
            s: 0xff,
            a: 0,
            x: 0,
            y: 0,
            p_carry: false,
            p_zero: false,
            p_interrupt_disable: true,
            p_decimal_mode: false,
            p_overflow: false,
            p_negative: false,
        }
    }

    pub fn get_p(&self) -> u8 {
        (if self.p_carry { P_CARRY_FLAG } else { 0 })
            | (if self.p_zero { P_ZERO_FLAG } else { 0 })
            | (if self.p_interrupt_disable {
                P_I_INTERRUPT_DISABLE_FLAG
            } else {
                0
            })
            | (if self.p_decimal_mode {
                P_DECIMAL_MODE_FLAG
            } else {
                0
            })
            | P_BIT_5_FLAG
            | (if self.p_overflow { P_OVERFLOW_FLAG } else { 0 })
            | (if self.p_negative { P_NEGATIVE_FLAG } else { 0 })
    }

    pub fn set_p(&mut self, p: u8) {
        self.p_carry = p & P_CARRY_FLAG != 0;
        self.p_zero = p & P_ZERO_FLAG != 0;
        self.p_interrupt_disable = p & P_I_INTERRUPT_DISABLE_FLAG != 0;
        self.p_decimal_mode = p & P_DECIMAL_MODE_FLAG != 0;
        self.p_overflow = p & P_OVERFLOW_FLAG != 0;
        self.p_negative = p & P_NEGATIVE_FLAG != 0;
    }

    pub fn set_p_zero_negative(&mut self, in_operand: u8) {
        self.p_zero = in_operand == 0;
        self.p_negative = (in_operand & 0b10000000) != 0;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
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
