pub const MAX_SOUND_REG_WRITES: usize = 450;

#[repr(C, packed)]
#[derive(Default)]
pub struct SoundRegWrite {
    pub cycle_offset: u16,
    pub data: u8,
}

#[repr(C, packed)]
pub struct SoundRegisterWrites {
    pub base_cycle_count: u64,
    pub num_entries: usize,
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

    pub fn push(&mut self, cycles: u64, data: u8) {
        let cycle_offset = cycles
            .checked_sub(self.base_cycle_count)
            .and_then(|offset| u16::try_from(offset).ok())
            .expect("Sound register write cycle offset is too large");

        if self.num_entries >= MAX_SOUND_REG_WRITES {
            panic!("Sound register write buffer is full");
        }

        self.entries[self.num_entries] = SoundRegWrite { cycle_offset, data };
        self.num_entries += 1;
    }
}
