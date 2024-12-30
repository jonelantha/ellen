use wasm_bindgen::prelude::*;

mod utils;

const RAM_SIZE: usize = 0x10000;

#[wasm_bindgen]
pub struct Ch22Memory {
    whole_ram: [u8; RAM_SIZE],
}

#[wasm_bindgen]
impl Ch22Memory {
    pub fn new() -> Ch22Memory {
        utils::set_panic_hook();

        Ch22Memory {
            whole_ram: [0; RAM_SIZE],
        }
    }

    pub fn whole_ram_start(&self) -> *const u8 {
        self.whole_ram.as_ptr()
    }

    pub fn whole_ram_size(&self) -> usize {
        RAM_SIZE
    }
}
