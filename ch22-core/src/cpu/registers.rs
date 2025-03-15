use super::util::*;

#[derive(Default)]
pub struct Registers {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub accumulator: u8,
    pub x_index: u8,
    pub y_index: u8,
    pub processor_flags: ProcessorFlags,
}

pub fn advance_program_counter(program_counter: &mut u16) {
    *program_counter = program_counter.wrapping_add(1);
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub struct ProcessorFlags {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal_mode: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl ProcessorFlags {
    pub fn update_zero_negative(&mut self, value: u8) {
        self.update_zero(value);
        self.update_negative(value);
    }

    pub fn update_zero(&mut self, value: u8) {
        self.zero = value == 0;
    }

    pub fn update_negative(&mut self, value: u8) {
        self.negative = is_negative(value);
    }
}

impl From<u8> for ProcessorFlags {
    fn from(flags: u8) -> Self {
        ProcessorFlags {
            carry: flags & P_CARRY != 0,
            zero: flags & P_ZERO != 0,
            interrupt_disable: flags & P_INTERRUPT_DISABLE != 0,
            decimal_mode: flags & P_DECIMAL_MODE != 0,
            overflow: flags & P_OVERFLOW != 0,
            negative: flags & P_NEGATIVE != 0,
        }
    }
}

impl From<ProcessorFlags> for u8 {
    fn from(
        ProcessorFlags {
            carry,
            zero,
            interrupt_disable,
            decimal_mode,
            overflow,
            negative,
        }: ProcessorFlags,
    ) -> Self {
        (if carry { P_CARRY } else { 0 })
            | (if zero { P_ZERO } else { 0 })
            | (if interrupt_disable {
                P_INTERRUPT_DISABLE
            } else {
                0
            })
            | (if decimal_mode { P_DECIMAL_MODE } else { 0 })
            | P_BIT_5
            | (if overflow { P_OVERFLOW } else { 0 })
            | (if negative { P_NEGATIVE } else { 0 })
    }
}

pub const P_CARRY: u8 = 0b00000001;
pub const P_ZERO: u8 = 0b00000010;
pub const P_INTERRUPT_DISABLE: u8 = 0b00000100;
pub const P_DECIMAL_MODE: u8 = 0b00001000;
pub const P_BREAK: u8 = 0b00010000;
pub const P_BIT_5: u8 = 0b00100000;
pub const P_OVERFLOW: u8 = 0b01000000;
pub const P_NEGATIVE: u8 = 0b10000000;
