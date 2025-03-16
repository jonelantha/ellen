use crate::bus::*;

pub struct StackAccess<'a, B: Bus> {
    bus: &'a mut B,
    stack_pointer: &'a mut u8,
}

impl<'a, B: Bus> StackAccess<'a, B> {
    pub fn new(bus: &'a mut B, stack_pointer: &'a mut u8) -> Self {
        StackAccess { bus, stack_pointer }
    }

    pub fn phantom_read(&mut self) {
        self.bus.phantom_read(0x100 + (*self.stack_pointer as u16));
    }

    pub fn push(&mut self, value: u8) {
        self.bus
            .write(0x100 + (*self.stack_pointer as u16), value, CycleOp::None);

        *self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn push_16(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.push(high);
        self.push(low);
    }

    pub fn pop(&mut self) -> u8 {
        *self.stack_pointer = self.stack_pointer.wrapping_add(1);

        self.bus
            .read(0x100 + (*self.stack_pointer as u16), CycleOp::None)
    }

    pub fn pop_16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }
}
