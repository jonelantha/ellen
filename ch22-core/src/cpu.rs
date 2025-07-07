use executor::*;
use interrupt_due_state::*;
use registers::*;

use crate::cpu_io::CpuIO;
use crate::word::Word;

pub mod executor;
pub mod interrupt_due_state;
pub mod registers;
pub mod util;

#[derive(Default)]
pub struct Cpu {
    registers: Registers,
    interrupt_due_state: InterruptDueState,
}

impl Cpu {
    pub fn reset<IO: CpuIO>(&mut self, io: &mut IO) {
        self.registers = Registers {
            program_counter: Word(
                io.read(RESET_VECTOR),
                io.read(RESET_VECTOR.same_page_add(1)),
            ),
            stack_pointer: 0xff,
            flags: ProcessorFlags {
                interrupt_disable: true,
                ..Default::default()
            },
            ..Default::default()
        };

        self.interrupt_due_state = InterruptDueState::default();
    }

    pub fn handle_next_instruction<IO: CpuIO>(&mut self, io: &mut IO) {
        execute(
            io,
            &mut self.registers,
            &mut self.interrupt_due_state,
            false,
        );
    }
}
