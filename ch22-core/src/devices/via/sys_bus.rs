use super::port_connections::{ViaPortConnections, ViaPortState};
use crate::sound::SoundRegisterWriteRecorder;

#[derive(Default)]
pub struct SysViaBus {
    sound: SoundRegisterWriteRecorder,
    ic32_latch: IC32Latch,
}

impl SysViaBus {
    /// IC32 outputs 4 and 5 are wired to the video address translation: bit 0
    /// of the result is output 4, bit 1 is output 5.
    pub fn video_base_bits(&self) -> u8 {
        (self.ic32_latch.get() >> 4) & 0b11
    }

    #[cfg(test)]
    pub fn ic32(&self) -> u8 {
        self.ic32_latch.get()
    }

    pub fn sound(&self) -> &SoundRegisterWriteRecorder {
        &self.sound
    }

    pub fn sound_mut(&mut self) -> &mut SoundRegisterWriteRecorder {
        &mut self.sound
    }
}

impl ViaPortConnections for SysViaBus {
    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64) {
        let port_a_data = port_a.resolve_floating();
        let port_b_data = port_b.resolve_floating();

        let latch_output = self.ic32_latch.update(port_b_data & 0x0f);

        let latch_output_line_0 = latch_output & 0x01 == 0x01;

        let sound_written = self.sound.update(latch_output_line_0, port_a_data, cycles);

        if sound_written {
            debug_check_sound_mask(port_a.output_mask, cycles);
        }
    }

    fn ic32_latch(&self) -> u8 {
        self.ic32_latch.get()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
fn debug_check_sound_mask(output_mask: u8, cycles: u64) {
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

#[derive(Default)]
struct IC32Latch {
    latch: u8,
}

impl IC32Latch {
    fn get(&self) -> u8 {
        self.latch
    }

    fn update(&mut self, data: u8) -> u8 {
        let bit = 1 << (data & 0x07);

        if data & 0x08 != 0 {
            self.latch |= bit;
        } else {
            self.latch &= !bit;
        }

        self.latch
    }
}
