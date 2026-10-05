#[cfg(test)]
mod tests;

pub const MAX_SOUND_REG_WRITES: usize = 500;

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
    pub entries: [SoundRegWrite; MAX_SOUND_REG_WRITES],
}

impl Default for SoundRegisterWrites {
    fn default() -> Self {
        Self {
            base_cycle_count: 0,
            num_entries: 0,
            entries: std::array::from_fn(|_| SoundRegWrite::default()),
        }
    }
}

impl SoundRegisterWrites {
    pub fn reset(&mut self, base_cycle_count: u64) {
        self.base_cycle_count = base_cycle_count;
        self.num_entries = 0;
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
    /// is beyond the 16 bit offset from the base) is ignored: the audio
    /// glitches for the rest of the field, then recovers. Builds with debug
    /// assertions panic instead, so it isn't missed.
    pub fn push(&mut self, cycles: u64, data: u8) {
        self.push_with(cycles, data, cfg!(debug_assertions));
    }

    /// `push`, but ignoring a write that doesn't fit in every build.
    #[cfg(test)]
    pub fn push_no_panic(&mut self, cycles: u64, data: u8) {
        self.push_with(cycles, data, false);
    }

    fn push_with(&mut self, cycles: u64, data: u8, panic_on_out_of_bounds: bool) {
        debug_assert!(
            self.base_cycle_count <= cycles,
            "Sound register write is before the base cycle count",
        );

        let Ok(cycle_offset) = u16::try_from(cycles - self.base_cycle_count) else {
            if panic_on_out_of_bounds {
                panic!("Sound register write cycle offset is too large");
            }
            return;
        };

        let index = self.num_entries as usize;

        if index >= MAX_SOUND_REG_WRITES {
            if panic_on_out_of_bounds {
                panic!("Sound register write buffer is full");
            }
            return;
        }

        self.entries[index] = SoundRegWrite { cycle_offset, data };
        self.num_entries += 1;
    }
}
