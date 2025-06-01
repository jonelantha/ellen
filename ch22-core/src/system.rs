use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::{cpu::Ch22Cpu, memory::Ch22Memory};

#[wasm_bindgen]
pub struct Ch22System {
    cpu: Ch22Cpu,
    memory: Ch22Memory,
}

#[wasm_bindgen]
impl Ch22System {
    pub fn new(
        js_get_irq_nmi: Function,
        js_do_phase_2: Function,
        js_wrap_counts: Function,
        js_is_io: Function,
        js_read_fallback: Function,
        js_write_fallback: Function,
    ) -> Ch22System {
        let get_irq_nmi = Box::new(move |machine_cycles: u32| {
            let flags = js_get_irq_nmi
                .call1(&JsValue::NULL, &machine_cycles.into())
                .expect("js_get_irq_nmi error")
                .as_f64()
                .expect("js_get_irq_nmi error") as u8;

            // irq, nmi
            (flags & 1 != 0, flags & 2 != 0)
        });

        let do_phase_2 = Box::new(move |machine_cycles: u32| {
            js_do_phase_2
                .call1(&JsValue::NULL, &machine_cycles.into())
                .expect("js_do_phase_2 error");
        });

        let wrap_counts = Box::new(move |wrap: u32| {
            js_wrap_counts
                .call1(&JsValue::NULL, &wrap.into())
                .expect("js_wrap_counts error");
        });

        let is_io = Box::new(move |address: u16| -> bool {
            js_is_io
                .call1(&JsValue::NULL, &address.into())
                .expect("js_is_io error")
                .as_bool()
                .expect("js_is_io error")
        });

        let read_fallback = Box::new(move |address: u16, machine_cycles: u32| -> u8 {
            js_read_fallback
                .call2(&JsValue::NULL, &address.into(), &machine_cycles.into())
                .expect("js_read_fallback error")
                .as_f64()
                .expect("js_read_fallback error") as u8
        });

        let write_fallback = Box::new(
            move |address: u16, value: u8, machine_cycles: u32| -> bool {
                js_write_fallback
                    .call3(
                        &JsValue::NULL,
                        &address.into(),
                        &value.into(),
                        &machine_cycles.into(),
                    )
                    .expect("js_write_fallback error")
                    .as_bool()
                    .expect("js_write_fallback error")
            },
        );

        Ch22System {
            cpu: Ch22Cpu::new(get_irq_nmi, do_phase_2, wrap_counts),
            memory: Ch22Memory::new(is_io, read_fallback, write_fallback),
        }
    }

    pub fn ram_start(&self) -> *const u8 {
        self.memory.ram_start()
    }

    pub fn ram_size(&self) -> usize {
        self.memory.ram_size()
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.memory);
    }

    pub fn handle_next_instruction(&mut self) -> u32 {
        self.cpu.handle_next_instruction(&mut self.memory)
    }
}
