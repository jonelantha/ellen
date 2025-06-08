use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::cpu::*;
use crate::cycle_manager::*;
use crate::js_device::*;
use crate::memory::*;
use crate::utils;

#[wasm_bindgen]
pub struct Ch22System {
    cpu: Ch22Cpu,
    cycle_manager: CycleManager,
}

#[wasm_bindgen]
impl Ch22System {
    pub fn new(
        js_get_irq_nmi: Function,
        js_wrap_counts: Function,
        js_read_fallback: Function,
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

        let read_fallback = Box::new(move |address: u16, cycles: u32| -> u8 {
            js_read_fallback
                .call2(&JsValue::NULL, &address.into(), &cycles.into())
                .expect("js_read_fallback error")
                .as_f64()
                .expect("js_read_fallback error") as u8
        });

        let cycle_manager =
            CycleManager::new(Ch22Memory::new(read_fallback), get_irq_nmi, wrap_counts);

        Ch22System {
            cpu: Ch22Cpu::new(),
            cycle_manager,
        }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.cycle_manager.device_list.memory.ram_start()
    }

    pub fn ram_size(&self) -> usize {
        self.cycle_manager.device_list.memory.ram_size()
    }

    pub fn reset(&mut self) {
        self.cpu.reset(
            self.cycle_manager.cycles,
            &mut self.cycle_manager.device_list.memory,
        );
    }

    pub fn handle_next_instruction(&mut self) -> u32 {
        self.cpu.handle_next_instruction(&mut self.cycle_manager);

        self.cycle_manager.cycles
    }

    pub fn add_device_js(
        &mut self,
        start_address: u16,
        end_address: u16,
        js_read: Function,
        js_write: Function,
        js_write_phase_2: Option<Function>,
        is_slow: bool,
    ) {
        self.cycle_manager.device_list.add_device(
            start_address..=end_address,
            Box::new(JsCh22Device::new(
                js_read,
                js_write,
                js_write_phase_2,
                is_slow,
            )),
        )
    }
}
