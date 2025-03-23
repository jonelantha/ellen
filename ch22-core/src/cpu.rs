use executor::*;
use js_sys::Function;
use registers::*;
use wasm_bindgen::prelude::*;
//use web_sys::console;

use crate::cycle_manager::*;
use crate::memory::*;
use crate::utils;
use crate::word::Word;

pub mod executor;
pub mod registers;
pub mod util;

#[wasm_bindgen]
pub struct Ch22Cpu {
    advance_cycles: Box<dyn Fn(u8, bool)>,
    registers: Registers,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new(js_advance_cycles: Function) -> Ch22Cpu {
        utils::set_panic_hook();

        let advance_cycles = Box::new(move |cycles: u8, check_interrupt: bool| {
            js_advance_cycles
                .call2(&JsValue::NULL, &cycles.into(), &check_interrupt.into())
                .expect("js_advance_cycles error");
        });

        Ch22Cpu {
            advance_cycles,
            registers: Registers::default(),
        }
    }

    pub fn reset(&mut self, memory: &mut Ch22Memory) -> bool {
        let vector: u16 = RESET_VECTOR.into();

        self.registers = Registers {
            program_counter: Word(memory.read(vector), memory.read(vector + 1)),
            stack_pointer: 0xff,
            flags: ProcessorFlags {
                interrupt_disable: true,
                ..Default::default()
            },
            ..Default::default()
        };

        self.registers.flags.interrupt_disable
    }

    pub fn handle_next_instruction(&mut self, memory: &mut Ch22Memory) -> bool {
        let mut cycle_manager = CycleManager::new(memory, &self.advance_cycles);

        execute(&mut cycle_manager, &mut self.registers, false);

        self.registers.flags.interrupt_disable
    }

    pub fn interrupt(&mut self, memory: &mut Ch22Memory, nmi: bool) -> bool {
        let mut cycle_manager = CycleManager::new(memory, &self.advance_cycles);

        interrupt(&mut cycle_manager, &mut self.registers, nmi);

        self.registers.flags.interrupt_disable
    }
}
