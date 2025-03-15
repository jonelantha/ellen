use crate::cpu::registers::*;
use crate::cycle_manager::*;

pub fn read_immediate_16<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    program_counter: &mut u16,
) -> u16 {
    u16::from_le_bytes([
        read_immediate(cycle_manager, program_counter),
        read_immediate(cycle_manager, program_counter),
    ])
}

pub fn read_immediate<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    program_counter: &mut u16,
) -> u8 {
    let value = cycle_manager.read(*program_counter, CycleOp::None);

    advance_program_counter(program_counter);

    value
}
