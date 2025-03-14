use crate::cycle_manager::{CycleManagerTrait, CycleOp};

pub struct MemoryAccess<'a, T: CycleManagerTrait + 'a> {
    cycle_manager: &'a mut T,
    pub program_counter: &'a mut u16,
    pub stack_pointer: &'a mut u8,
}

impl<'a, T: CycleManagerTrait + 'a> MemoryAccess<'a, T> {
    pub fn new(
        cycle_manager: &'a mut T,
        program_counter: &'a mut u16,
        stack_pointer: &'a mut u8,
    ) -> Self {
        MemoryAccess {
            cycle_manager,
            program_counter,
            stack_pointer,
        }
    }

    pub fn phantom_read(&mut self, address: u16) {
        self.cycle_manager.phantom_read(address);
    }

    pub fn read(&mut self, address: u16, op: CycleOp) -> u8 {
        self.cycle_manager.read(address, op)
    }

    pub fn write(&mut self, address: u16, value: u8, op: CycleOp) {
        self.cycle_manager.write(address, value, op);
    }

    pub fn complete_instruction(&mut self) {
        self.cycle_manager.complete();
    }

    pub fn read_16(&mut self, address: u16, op: CycleOp) -> u16 {
        u16::from_le_bytes([
            self.read(address, op),
            self.read(next_address_same_page(address), op),
        ])
    }

    pub fn phantom_program_counter_read(&mut self) {
        self.phantom_read(*self.program_counter);
    }

    pub fn increment_program_counter(&mut self) {
        *self.program_counter = self.program_counter.wrapping_add(1);
    }

    pub fn read_immediate(&mut self) -> u8 {
        let value = self.read(*self.program_counter, CycleOp::None);

        self.increment_program_counter();

        value
    }

    pub fn read_immediate_16(&mut self) -> u16 {
        u16::from_le_bytes([self.read_immediate(), self.read_immediate()])
    }

    pub fn push(&mut self, value: u8) {
        self.write(0x100 + (*self.stack_pointer as u16), value, CycleOp::None);

        *self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn pop(&mut self) -> u8 {
        *self.stack_pointer = self.stack_pointer.wrapping_add(1);

        self.read(0x100 + (*self.stack_pointer as u16), CycleOp::None)
    }

    pub fn phantom_stack_read(&mut self) {
        self.phantom_read(0x100 + (*self.stack_pointer as u16));
    }

    pub fn push_16(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.push(high);
        self.push(low);
    }

    pub fn pop_16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    pub fn push_program_counter(&mut self) {
        self.push_16(*self.program_counter);
    }

    pub fn pop_program_counter(&mut self) {
        *self.program_counter = self.pop_16();
    }
}

fn next_address_same_page(address: u16) -> u16 {
    let [address_low, address_high] = address.to_le_bytes();

    u16::from_le_bytes([address_low.wrapping_add(1), address_high])
}
