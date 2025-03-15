use crate::cycle_manager::{CycleManagerTrait, CycleOp};

pub fn phantom_stack_read<T: CycleManagerTrait>(cycle_manager: &mut T, stack_pointer: u8) {
    cycle_manager.phantom_read(0x100 + (stack_pointer as u16));
}

pub fn push<T: CycleManagerTrait>(cycle_manager: &mut T, stack_pointer: &mut u8, value: u8) {
    cycle_manager.write(0x100 + (*stack_pointer as u16), value, CycleOp::None);

    *stack_pointer = stack_pointer.wrapping_sub(1);
}

pub fn push_16<T: CycleManagerTrait>(cycle_manager: &mut T, stack_pointer: &mut u8, value: u16) {
    let [low, high] = value.to_le_bytes();
    push(cycle_manager, stack_pointer, high);
    push(cycle_manager, stack_pointer, low);
}

pub fn pop<T: CycleManagerTrait>(cycle_manager: &mut T, stack_pointer: &mut u8) -> u8 {
    *stack_pointer = stack_pointer.wrapping_add(1);

    cycle_manager.read(0x100 + (*stack_pointer as u16), CycleOp::None)
}

pub fn pop_16<T: CycleManagerTrait>(cycle_manager: &mut T, stack_pointer: &mut u8) -> u16 {
    u16::from_le_bytes([
        pop(cycle_manager, stack_pointer),
        pop(cycle_manager, stack_pointer),
    ])
}
