use crate::bus::*;
use crate::cpu::registers::*;

pub fn read_immediate_16<B: Bus>(bus: &mut B, program_counter: &mut u16) -> u16 {
    u16::from_le_bytes([
        read_immediate(bus, program_counter),
        read_immediate(bus, program_counter),
    ])
}

pub fn read_immediate<B: Bus>(bus: &mut B, program_counter: &mut u16) -> u8 {
    let value = bus.read(*program_counter, CycleOp::None);

    advance_program_counter(program_counter);

    value
}
