use super::super::cpu_io::CpuIO;
use super::super::interrupt_due_state::*;
use crate::word::*;

pub fn phantom_stack_read<IO: CpuIO>(io: &mut IO, stack_pointer: u8) {
    io.phantom_read(Word::stack_page(stack_pointer));
}

pub fn push<IO: CpuIO>(io: &mut IO, stack_pointer: &mut u8, value: u8) {
    io.write(Word::stack_page(*stack_pointer), value);

    *stack_pointer = stack_pointer.wrapping_sub(1);
}

pub fn push_word<IO: CpuIO>(io: &mut IO, stack_pointer: &mut u8, Word(low, high): Word) {
    push(io, stack_pointer, high);
    push(io, stack_pointer, low);
}

pub fn pop<IO: CpuIO>(io: &mut IO, stack_pointer: &mut u8) -> u8 {
    *stack_pointer = stack_pointer.wrapping_add(1);

    io.read(Word::stack_page(*stack_pointer))
}

pub fn pop_word<IO: CpuIO>(io: &mut IO, stack_pointer: &mut u8) -> Word {
    Word(pop(io, stack_pointer), pop(io, stack_pointer))
}

pub fn pop_word_with_interrupt_check<IO: CpuIO>(
    io: &mut IO,
    stack_pointer: &mut u8,
    interrupt_disable: bool,
    interrupt_due_state: &mut InterruptDueState,
) -> Word {
    let low = pop(io, stack_pointer);

    update_interrupt_due_state(interrupt_due_state, io, interrupt_disable);

    let high = pop(io, stack_pointer);

    Word(low, high)
}

pub fn immediate_fetch<IO: CpuIO>(io: &mut IO, program_counter: &mut Word) -> u8 {
    let value = io.read(*program_counter);

    program_counter.increment();

    value
}

pub fn immediate_fetch_word<IO: CpuIO>(io: &mut IO, program_counter: &mut Word) -> Word {
    Word(
        immediate_fetch(io, program_counter),
        immediate_fetch(io, program_counter),
    )
}

pub fn immediate_fetch_word_with_interrupt_check<IO: CpuIO>(
    io: &mut IO,
    program_counter: &mut Word,
    interrupt_disable: bool,
    interrupt_due_state: &mut InterruptDueState,
) -> Word {
    let low = immediate_fetch(io, program_counter);

    update_interrupt_due_state(interrupt_due_state, io, interrupt_disable);

    let high = immediate_fetch(io, program_counter);

    Word(low, high)
}

pub fn read_word<IO: CpuIO>(io: &mut IO, address: Word) -> Word {
    Word(io.read(address), io.read(address.same_page_add(1)))
}

pub fn read_word_with_interrupt_check<IO: CpuIO>(
    io: &mut IO,
    address: Word,
    interrupt_disable: bool,
    interrupt_due_state: &mut InterruptDueState,
) -> Word {
    let low = io.read(address);

    update_interrupt_due_state(interrupt_due_state, io, interrupt_disable);

    let high = io.read(address.same_page_add(1));

    Word(low, high)
}
