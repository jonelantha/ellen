use super::port_connections::{ViaPortConnections, ViaPortState};
use crate::sound::SoundRegisterWriteRecorder;
use crate::video::VideoBase;

#[derive(Default)]
pub struct SysViaBus {
    sound: SoundRegisterWriteRecorder,
    ic32_latch: IC32Latch,
}

impl SysViaBus {
    /// IC32 outputs 5 and 4 are wired to the video address translation.
    pub fn video_base(&self) -> VideoBase {
        let latch = self.ic32_latch.get();

        VideoBase::from_bits((latch & 0b0010_0000 != 0, latch & 0b0001_0000 != 0))
    }

    /// The raw latch. IC32 bits 1 to 3, 6 and 7 drive nothing yet, so tests of
    /// the latch have no output to observe them through; once they do, those
    /// tests should assert on the outputs instead.
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

/// Logs sound writes made while port A is not fully driven, in debug wasm
/// builds only. The write at cycle 0 is skipped: it is the power-on one, made
/// with the ports undriven.
fn debug_check_sound_mask(output_mask: u8, cycles: u64) {
    if cfg!(all(target_arch = "wasm32", debug_assertions)) && output_mask != 0xff && cycles != 0 {
        web_sys::console::log_1(
            &format!("sound write output_mask == {output_mask:#04x} {cycles:?}").into(),
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
