use crate::sound_register_writes::SoundRegisterWrites;

/// Turns the levels on a sound chip's /WE and 8 bit data inputs into register
/// writes, recorded for the field in progress into a buffer that is read from
/// outside (see `SoundRegisterWrites`). Nothing here depends on which chip is
/// attached, other than the timing assumptions noted on `update`.
#[derive(Default)]
pub struct SoundRegisterWriteRecorder {
    sound_register_writes: SoundRegisterWrites,
    previous_data: Option<u8>,
}

impl SoundRegisterWriteRecorder {
    /// Empties the buffer; recorded cycle offsets are relative to `base_cycle_count`.
    pub fn start_field(&mut self, base_cycle_count: u64) {
        self.sound_register_writes.reset(base_cycle_count);
    }

    pub fn register_writes_ptr(&self) -> *const SoundRegisterWrites {
        &raw const self.sound_register_writes
    }

    #[cfg(test)]
    pub fn register_writes(&self) -> &SoundRegisterWrites {
        &self.sound_register_writes
    }

    /// Called with the current level of the chip's inputs whenever anything
    /// upstream may have changed them, so most calls change nothing.
    ///
    /// Records one write per /WE fall, plus one for each later change of the
    /// data while /WE is low. /WE rising is not recorded.
    ///
    /// An SN76489 samples the bus about 32 chip clocks (16 cycles) after
    /// /WE falls. These cases are not modelled right now:
    /// - /WE falling again inside the window
    /// - data changing in the 32 cycle window after /WE is high
    /// - /WE held low for long periods with the possibility of repeated writes
    ///
    /// These cases have not yet (significantly) been observed in real code
    ///
    /// Returns true if this update latched a register write.
    pub fn update(&mut self, write_enable_active_low: bool, data: u8, cycles: u64) -> bool {
        if write_enable_active_low {
            self.previous_data = None;
            return false;
        }

        if self.previous_data == Some(data) {
            return false;
        }

        self.sound_register_writes.push(cycles, data);
        self.previous_data = Some(data);

        true
    }
}
