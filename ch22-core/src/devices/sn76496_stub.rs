use crate::sound_register_writes::SoundRegisterWrites;

pub trait SoundChip {
    /// Called with the current level of the chip's inputs whenever anything
    /// upstream may have changed them, so most calls change nothing.
    ///
    /// The chip only sees input levels, and right now only logs writes when
    /// enabled and something changes - in the future we move to a full event
    /// logging model (logging disabled events) and let the renderer decide
    /// when a write occured or reoccured
    ///
    /// Returns true if this update latched a register write.
    fn update(&mut self, write_enable_active_low: bool, data: u8, cycles: u64) -> bool;
}

/// Records the chip's register writes for the field in progress into a buffer
/// that is read from outside (see `SoundRegisterWrites`).
#[derive(Default)]
pub struct SN76496Stub {
    sound_register_writes: SoundRegisterWrites,
    previous_data: Option<u8>,
}

impl SN76496Stub {
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
}

impl SoundChip for SN76496Stub {
    fn update(&mut self, write_enable_active_low: bool, data: u8, cycles: u64) -> bool {
        if !write_enable_active_low {
            if self
                .previous_data
                .is_none_or(|previous_data| previous_data != data)
            {
                self.sound_register_writes.push(cycles, data);

                self.previous_data = Some(data);

                return true;
            }
        } else {
            self.previous_data = None;
        }

        false
    }
}
