use executor::*;
use interrupt_due_state::*;
use registers::*;

use crate::cycle_manager::*;
use crate::memory::*;
use crate::utils;
use crate::word::Word;

pub mod executor;
pub mod interrupt_due_state;
pub mod registers;
pub mod util;

const CYCLE_COUNT_WRAP: u32 = 0x3FFFFFFF;

pub struct Ch22Cpu {
    get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
    do_phase_2: Box<dyn Fn(u32)>,
    wrap_counts: Box<dyn Fn(u32)>,
    registers: Registers,
    interrupt_due_state: InterruptDueState,
    machine_cycles: u32,
}

impl Ch22Cpu {
    pub fn new(
        get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
        do_phase_2: Box<dyn Fn(u32)>,
        wrap_counts: Box<dyn Fn(u32)>,
    ) -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu {
            machine_cycles: 0,
            get_irq_nmi,
            do_phase_2,
            wrap_counts,
            registers: Registers::default(),
            interrupt_due_state: InterruptDueState::default(),
        }
    }

    pub fn reset(&mut self, memory: &mut Ch22Memory) {
        let vector: u16 = RESET_VECTOR.into();

        self.registers = Registers {
            program_counter: Word(
                memory.read(vector, self.machine_cycles),
                memory.read(vector + 1, self.machine_cycles),
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

    pub fn handle_next_instruction(&mut self, memory: &mut Ch22Memory) -> u32 {
        let mut cycle_manager = CycleManager::new(
            memory,
            &mut self.machine_cycles,
            &self.get_irq_nmi,
            &self.do_phase_2,
        );

        execute(
            &mut cycle_manager,
            &mut self.registers,
            &mut self.interrupt_due_state,
            false,
        );

        if self.machine_cycles > CYCLE_COUNT_WRAP {
            (self.wrap_counts)(CYCLE_COUNT_WRAP);
            self.machine_cycles -= CYCLE_COUNT_WRAP;
        }

        self.machine_cycles
    }
}
