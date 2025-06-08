use executor::*;
use interrupt_due_state::*;
use registers::*;

use crate::cpu_io::CpuIO;
use crate::device::*;
use crate::memory::*;
use crate::utils;
use crate::word::Word;

pub mod executor;
pub mod interrupt_due_state;
pub mod registers;
pub mod util;

#[derive(Default)]
pub struct Ch22Cpu {
    registers: Registers,
    interrupt_due_state: InterruptDueState,
}

impl Ch22Cpu {
    pub fn new() -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu::default()
    }

    pub fn reset(&mut self, machine_cycles: u32, memory: &mut Ch22Memory) {
        let vector: u16 = RESET_VECTOR.into();

        self.registers = Registers {
            program_counter: Word(
                memory.read(vector, machine_cycles),
                memory.read(vector + 1, machine_cycles),
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
