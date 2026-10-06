#[cfg(test)]
mod tests;

pub const MAX_SOUND_REG_WRITES: usize = 500;

/// `SoundRegisterWrites::dropped` bits, set when a write did not fit.
pub const DROPPED_BUFFER_FULL: u8 = 0b01;
pub const DROPPED_OFFSET_TOO_LARGE: u8 = 0b10;

#[repr(C, packed)]
#[derive(Default)]
pub struct SoundRegWrite {
    pub cycle_offset: u16,
    pub data: u8,
}

#[repr(C, packed)]
pub struct SoundRegisterWrites {
    pub base_cycle_count: u64,
    pub num_entries: u32,
    pub dropped: u8,
    pub entries: [SoundRegWrite; MAX_SOUND_REG_WRITES],
}

impl Default for SoundRegisterWrites {
    fn default() -> Self {
        Self {
            base_cycle_count: 0,
            num_entries: 0,
            dropped: 0,
            entries: std::array::from_fn(|_| SoundRegWrite::default()),
        }
    }
}

impl SoundRegisterWrites {
    pub fn reset(&mut self, base_cycle_count: u64) {
        self.base_cycle_count = base_cycle_count;
        self.num_entries = 0;
        self.dropped = 0;
    }

    /// The recorded writes as (cycle, data).
    #[cfg(test)]
    pub fn cycles_and_data(&self) -> Vec<(u64, u8)> {
        let base_cycle_count = self.base_cycle_count;

        self.entries[..self.num_entries as usize]
            .iter()
            .map(|entry| (base_cycle_count + u64::from(entry.cycle_offset), entry.data))
            .collect()
    }

    /// Records a write. One that doesn't fit (the buffer is full, or the cycle
    /// is beyond the 16 bit offset from the base) is ignored and flagged in
    /// `dropped`. Later writes in the field may still be recorded, but a
    /// channel keeps whatever its last recorded write set until it is
    /// written again.
    pub fn push(&mut self, cycles: u64, data: u8) {
        // The clock only moves forward, so a write is never before the field's base.
        debug_assert!(
            self.base_cycle_count <= cycles,
            "Sound register write is before the base cycle count",
        );

        let Ok(cycle_offset) = u16::try_from(cycles - self.base_cycle_count) else {
            self.dropped |= DROPPED_OFFSET_TOO_LARGE;
            return;
        };

        let index = self.num_entries as usize;

        if index >= MAX_SOUND_REG_WRITES {
            self.dropped |= DROPPED_BUFFER_FULL;
            return;
        }

        self.entries[index] = SoundRegWrite { cycle_offset, data };
        self.num_entries += 1;
    }
}
