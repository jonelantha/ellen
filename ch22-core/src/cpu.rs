use executor::*;
use interrupt_due_state::*;
use js_sys::Function;
use registers::*;
use wasm_bindgen::prelude::*;

use crate::cycle_manager::*;
use crate::memory::*;
use crate::utils;
use crate::word::Word;

pub mod executor;
pub mod interrupt_due_state;
pub mod registers;
pub mod util;

#[wasm_bindgen]
pub struct Ch22Cpu {
    get_irq_nmi: Box<dyn Fn(u8) -> (bool, bool)>,
    do_phase_2: Box<dyn Fn(u8)>,
    registers: Registers,
    interrupt_due_state: InterruptDueState,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new(js_get_irq_nmi: Function, js_do_phase_2: Function) -> Ch22Cpu {
        utils::set_panic_hook();

        let get_irq_nmi = Box::new(move |cycles: u8| {
            let flags = js_get_irq_nmi
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_get_irq_nmi error")
                .as_f64()
                .expect("js_get_irq_nmi error") as u8;

            // irq, nmi
            (flags & 1 != 0, flags & 2 != 0)
        });

        let do_phase_2 = Box::new(move |cycles: u8| {
            js_do_phase_2
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_do_phase_2 error");
        });

        Ch22Cpu {
            get_irq_nmi,
            do_phase_2,
            registers: Registers::default(),
            interrupt_due_state: InterruptDueState::default(),
        }
    }

    pub fn reset(&mut self, memory: &mut Ch22Memory) {
        let vector: u16 = RESET_VECTOR.into();

        self.registers = Registers {
            program_counter: Word(memory.read(vector, 0), memory.read(vector + 1, 0)),
            stack_pointer: 0xff,
            flags: ProcessorFlags {
                interrupt_disable: true,
                ..Default::default()
            },
            ..Default::default()
        };

        self.interrupt_due_state = InterruptDueState::default();
    }

    pub fn handle_next_instruction(&mut self, memory: &mut Ch22Memory) -> u8 {
        let mut cycle_manager = CycleManager::new(memory, &self.get_irq_nmi, &self.do_phase_2);

        execute(
            &mut cycle_manager,
            &mut self.registers,
            &mut self.interrupt_due_state,
            false,
        );

        cycle_manager.cycles
    }
}
