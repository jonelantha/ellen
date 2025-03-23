use crate::bus::*;
use crate::word::*;

pub fn phantom_stack_read<B: Bus>(bus: &mut B, stack_pointer: u8) {
    bus.phantom_read(Word::stack_page(stack_pointer));
}

pub fn push<B: Bus>(bus: &mut B, stack_pointer: &mut u8, value: u8) {
    bus.write(Word::stack_page(*stack_pointer), value, CycleOp::None);

    *stack_pointer = stack_pointer.wrapping_sub(1);
}

pub fn push_word<B: Bus>(bus: &mut B, stack_pointer: &mut u8, Word(low, high): Word) {
    push(bus, stack_pointer, high);
    push(bus, stack_pointer, low);
}

pub fn pop<B: Bus>(bus: &mut B, stack_pointer: &mut u8) -> u8 {
    *stack_pointer = stack_pointer.wrapping_add(1);

    bus.read(Word::stack_page(*stack_pointer), CycleOp::None)
}

pub fn pop_word<B: Bus>(bus: &mut B, stack_pointer: &mut u8) -> Word {
    Word(pop(bus, stack_pointer), pop(bus, stack_pointer))
}

pub fn immediate_fetch<B: Bus>(bus: &mut B, program_counter: &mut Word) -> u8 {
    let value = bus.read(*program_counter, CycleOp::None);

    program_counter.increment();

    value
}

pub fn immediate_fetch_word<B: Bus>(bus: &mut B, program_counter: &mut Word) -> Word {
    Word(
        immediate_fetch(bus, program_counter),
        immediate_fetch(bus, program_counter),
    )
}

pub fn read_word<B: Bus>(bus: &mut B, address: Word, op: CycleOp) -> Word {
    Word(
        bus.read(address, op),
        bus.read(address.same_page_add(1), op),
    )
}
