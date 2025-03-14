use crate::cycle_manager::{CycleManagerTrait, CycleOp};

pub struct MemoryAccess<'a, T: CycleManagerTrait + 'a> {
    cycle_manager: &'a mut T,
    pub program_counter: &'a mut u16,
}

impl<'a, T: CycleManagerTrait + 'a> MemoryAccess<'a, T> {
    pub fn new(cycle_manager: &'a mut T, program_counter: &'a mut u16) -> Self {
        MemoryAccess {
            cycle_manager,
            program_counter,
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
}

fn next_address_same_page(address: u16) -> u16 {
    let [address_low, address_high] = address.to_le_bytes();

    u16::from_le_bytes([address_low.wrapping_add(1), address_high])
}
