use crate::bus::*;
use crate::memory::*;
use crate::word::Word;

pub struct CycleManager<'a> {
    cycles: u8,
    memory: &'a mut Ch22Memory,
    advance_cycles_handler: &'a (dyn Fn(u8, bool) + 'a),
}

impl<'a> CycleManager<'a> {
    pub fn new(
        memory: &'a mut Ch22Memory,
        advance_cycles_handler: &'a (dyn Fn(u8, bool) + 'a),
    ) -> Self {
        CycleManager {
            cycles: 0,
            memory,
            advance_cycles_handler,
        }
    }
}

impl Bus for CycleManager<'_> {
    fn phantom_read(&mut self, _address: Word) {
        self.cycles += 1;
    }

    fn read(&mut self, address: Word, op: CycleOp) -> u8 {
        self.process_op(op);

        let value = self.memory.read(address.into());

        self.cycles += 1;

        value
    }

    fn write(&mut self, address: Word, value: u8, op: CycleOp) {
        self.process_op(op);

        self.memory.write(address.into(), value);

        self.cycles += 1;
    }

    fn complete(&self) {
        (self.advance_cycles_handler)(self.cycles, false);
    }
}

impl CycleManager<'_> {
    fn process_op(&mut self, op: CycleOp) {
        if op == CycleOp::Sync || op == CycleOp::CheckInterrupt {
            (self.advance_cycles_handler)(self.cycles, op == CycleOp::CheckInterrupt);

            self.cycles = 0;
        }
    }
}
