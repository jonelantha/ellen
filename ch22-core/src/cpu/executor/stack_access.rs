use crate::cycle_manager::{CycleManagerTrait, CycleOp};

use super::memory_access::MemoryAccess;

pub fn phantom_stack_read<T: CycleManagerTrait>(
    memory_access: &mut MemoryAccess<T>,
    stack_pointer: u8,
) {
    memory_access.phantom_read(0x100 + (stack_pointer as u16));
}

pub fn push<T: CycleManagerTrait>(
    memory_access: &mut MemoryAccess<T>,
    stack_pointer: &mut u8,
    value: u8,
) {
    memory_access.write(0x100 + (*stack_pointer as u16), value, CycleOp::None);

    *stack_pointer = stack_pointer.wrapping_sub(1);
}

pub fn push_16<T: CycleManagerTrait>(
    memory_access: &mut MemoryAccess<T>,
    stack_pointer: &mut u8,
    value: u16,
) {
    let [low, high] = value.to_le_bytes();
    push(memory_access, stack_pointer, high);
    push(memory_access, stack_pointer, low);
}

pub fn pop<T: CycleManagerTrait>(
    memory_access: &mut MemoryAccess<T>,
    stack_pointer: &mut u8,
) -> u8 {
    *stack_pointer = stack_pointer.wrapping_add(1);

    memory_access.read(0x100 + (*stack_pointer as u16), CycleOp::None)
}

pub fn pop_16<T: CycleManagerTrait>(
    memory_access: &mut MemoryAccess<T>,
    stack_pointer: &mut u8,
) -> u16 {
    u16::from_le_bytes([
        pop(memory_access, stack_pointer),
        pop(memory_access, stack_pointer),
    ])
}
