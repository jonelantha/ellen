use crate::bus::*;
use crate::word::*;

pub struct StackAccess<'a, B: Bus> {
    bus: &'a mut B,
    stack_pointer: &'a mut u8,
}

impl<'a, B: Bus> StackAccess<'a, B> {
    pub fn new(bus: &'a mut B, stack_pointer: &'a mut u8) -> Self {
        StackAccess { bus, stack_pointer }
    }

    pub fn phantom_read(&mut self) {
        self.bus.phantom_read(Word::stack_page(*self.stack_pointer));
    }

    pub fn push(&mut self, value: u8) {
        self.bus
            .write(Word::stack_page(*self.stack_pointer), value, CycleOp::None);

        *self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn push_word(&mut self, value: Word) {
        self.push(value.high);
        self.push(value.low);
    }

    pub fn pop(&mut self) -> u8 {
        *self.stack_pointer = self.stack_pointer.wrapping_add(1);

        self.bus
            .read(Word::stack_page(*self.stack_pointer), CycleOp::None)
    }

    pub fn pop_word(&mut self) -> Word {
        Word {
            low: self.pop(),
            high: self.pop(),
        }
    }
}
