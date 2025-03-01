use executor::Executor;
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
    js_advance_cycles: Function,
}

#[wasm_bindgen]
impl Ch22Cpu {
    pub fn new(js_advance_cycles: Function) -> Ch22Cpu {
        utils::set_panic_hook();

        Ch22Cpu { js_advance_cycles }
    }

    fn handle_advance_cycles(&self, cycles: u8, check_interrupt: bool) {
        self.js_advance_cycles
            .call2(&JsValue::NULL, &cycles.into(), &check_interrupt.into())
            .expect("js_advance_cycles error");
    }

    pub fn handle_next_instruction(&mut self, memory: &mut Ch22Memory, registers: &mut Registers) {
        let mut cycle_manager = CycleManager::new(
            memory,
            Box::new(|cycles, check_interrupt| self.handle_advance_cycles(cycles, check_interrupt)),
        );

        let mut executor = Executor::new(&mut cycle_manager, registers);

        executor.execute(false)
    }

    pub fn interrupt(&mut self, memory: &mut Ch22Memory, registers: &mut Registers, nmi: bool) {
        let mut cycle_manager = CycleManager::new(
            memory,
            Box::new(|cycles, check_interrupt| self.handle_advance_cycles(cycles, check_interrupt)),
        );

        let mut executor = Executor::new(&mut cycle_manager, registers);

        executor.interrupt(nmi)
    }
}
