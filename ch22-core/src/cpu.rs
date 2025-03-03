use executor::*;
use js_sys::Function;
use registers::Registers;
use wasm_bindgen::prelude::*;
//use web_sys::console;

use crate::cycle_manager::*;
use crate::memory::*;
use crate::utils;

pub mod executor;
pub mod registers;

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
            registers: Registers::new(),
        }
    }

    pub fn reset(&mut self, memory: &mut Ch22Memory) {
        self.registers.pc =
            u16::from_le_bytes([memory.read(RESET_VECTOR), memory.read(RESET_VECTOR + 1)]);
    }

    pub fn handle_next_instruction(&mut self, memory: &mut Ch22Memory) -> bool {
        let mut cycle_manager = CycleManager::new(memory, &self.advance_cycles);

        let mut executor = Executor::new(&mut cycle_manager, &mut self.registers);

        executor.execute(false);

        self.registers.p_interrupt_disable
    }

    pub fn interrupt(&mut self, memory: &mut Ch22Memory, nmi: bool) -> bool {
        let mut cycle_manager = CycleManager::new(memory, &self.advance_cycles);

        let mut executor = Executor::new(&mut cycle_manager, &mut self.registers);

        executor.interrupt(nmi);

        self.registers.p_interrupt_disable
    }
}
