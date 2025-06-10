use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::cpu::*;
use crate::cycle_manager::*;
use crate::device::io_space::*;
use crate::device::paged_rom::*;
use crate::device::ram::*;
use crate::device::rom::*;
use crate::utils;

#[wasm_bindgen]
pub struct Ch22System {
    cpu: Ch22Cpu,
    cycle_manager: CycleManager,
}

#[wasm_bindgen]
impl Ch22System {
    pub fn new(
        ram: Ch22Ram,
        paged_rom: Ch22PagedRom,
        io_space: Ch22IOSpace,
        rom: Ch22Rom,
        js_get_irq_nmi: Function,
        js_wrap_counts: Function,
    ) -> Ch22System {
        utils::set_panic_hook();

        let get_irq_nmi = Box::new(move |cycles: u32| {
            let flags = js_get_irq_nmi
                .call1(&JsValue::NULL, &cycles.into())
                .expect("js_get_irq_nmi error")
                .as_f64()
                .expect("js_get_irq_nmi error") as u8;

            // irq, nmi
            (flags & 1 != 0, flags & 2 != 0)
        });

        let wrap_counts = Box::new(move |wrap: u32| {
            js_wrap_counts
                .call1(&JsValue::NULL, &wrap.into())
                .expect("js_wrap_counts error");
        });

        let cycle_manager =
            CycleManager::new(ram, paged_rom, io_space, rom, get_irq_nmi, wrap_counts);

        Ch22System {
            cpu: Ch22Cpu::new(),
            cycle_manager,
        }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.cycle_manager.device_list.ram.ram_start()
    }

    pub fn ram_size(&self) -> usize {
        self.cycle_manager.device_list.ram.ram_size()
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.cycle_manager);
    }

    pub fn handle_next_instruction(&mut self) -> u32 {
        self.cpu.handle_next_instruction(&mut self.cycle_manager);

        self.cycle_manager.cycles
    }
}
