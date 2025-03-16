use crate::bus::*;

pub fn phantom_stack_read<T: BusTrait>(bus: &mut T, stack_pointer: u8) {
    bus.phantom_read(0x100 + (stack_pointer as u16));
}

pub fn push<T: BusTrait>(bus: &mut T, stack_pointer: &mut u8, value: u8) {
    bus.write(0x100 + (*stack_pointer as u16), value, CycleOp::None);

    *stack_pointer = stack_pointer.wrapping_sub(1);
}

pub fn push_16<T: BusTrait>(bus: &mut T, stack_pointer: &mut u8, value: u16) {
    let [low, high] = value.to_le_bytes();
    push(bus, stack_pointer, high);
    push(bus, stack_pointer, low);
}

pub fn pop<T: BusTrait>(bus: &mut T, stack_pointer: &mut u8) -> u8 {
    *stack_pointer = stack_pointer.wrapping_add(1);

    bus.read(0x100 + (*stack_pointer as u16), CycleOp::None)
}

pub fn pop_16<T: BusTrait>(bus: &mut T, stack_pointer: &mut u8) -> u16 {
    u16::from_le_bytes([pop(bus, stack_pointer), pop(bus, stack_pointer)])
}
