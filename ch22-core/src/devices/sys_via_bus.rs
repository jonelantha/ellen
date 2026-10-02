use std::{cell::Cell, rc::Rc};

use super::sn76496_stub::SoundChip;
use super::via_port_connections::{ViaPortConnections, ViaPortState};

pub struct SysViaBus<Sound> {
    sound: Sound,
    ic32_latch: IC32Latch,
}

impl<Sound: SoundChip> SysViaBus<Sound> {
    pub fn new(sound: Sound, ic32_latch: Rc<Cell<u8>>) -> Self {
        SysViaBus {
            sound,
            ic32_latch: IC32Latch::new(ic32_latch),
        }
    }

    fn debug_check_sound_mask(&self, output_mask: u8, cycles: u64) {
        if output_mask != 0xff {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!(
                    "sound write output_mask == {:#04x} {:?}",
                    output_mask, cycles
                )
                .into(),
            );
        }
    }
}

impl<Sound: SoundChip> ViaPortConnections for SysViaBus<Sound> {
    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64) {
        let port_a_data = port_a.resolve_floating();
        let port_b_data = port_b.resolve_floating();

        let latch_output = self.ic32_latch.update(port_b_data & 0x0f);

        let latch_output_line_0 = latch_output & 0x01 == 0x01;

        let sound_written = self.sound.update(latch_output_line_0, port_a_data, cycles);

        if sound_written {
            self.debug_check_sound_mask(port_a.output_mask, cycles);
        }
    }
}

struct IC32Latch {
    latch: Rc<Cell<u8>>,
}

impl IC32Latch {
    fn new(latch: Rc<Cell<u8>>) -> Self {
        IC32Latch { latch }
    }

    fn update(&mut self, data: u8) -> u8 {
        let bit = 1 << (data & 0x07);

        if data & 0x08 != 0 {
            self.latch.set(self.latch.get() | bit);
        } else {
            self.latch.set(self.latch.get() & !bit);
        };

        self.latch.get()
    }
}
