#[derive(Clone, Copy, Default)]
pub struct Word {
    pub low: u8,
    pub high: u8,
}

impl From<Word> for u16 {
    fn from(Word { low, high }: Word) -> Self {
        u16::from_le_bytes([low, high])
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        let [low, high] = u16::to_le_bytes(value);

        Word { low, high }
    }
}

impl Word {
    pub fn zero_page(zero_page_address: u8) -> Self {
        Word {
            high: 0,
            low: zero_page_address,
        }
    }

    pub fn stack_page(stack_pointer: u8) -> Self {
        Word {
            high: 1,
            low: stack_pointer,
        }
    }

    pub fn same_page_add(&self, offset: u8) -> Self {
        Word {
            high: self.high,
            low: self.low.wrapping_add(offset),
        }
    }

    pub fn paged_add(&self, offset: u8) -> (Word, OffsetResult) {
        let Word { low, high } = *self;

        let (low, carried) = low.overflowing_add(offset);

        if carried {
            let intermediate = Word { low, high };

            let high = high.wrapping_add(1);

            (Word { low, high }, OffsetResult::CrossedPage(intermediate))
        } else {
            (Word { low, high }, OffsetResult::SamePage)
        }
    }

    pub fn paged_subtract(&self, offset: u8) -> (Word, OffsetResult) {
        let Word { low, high } = *self;

        let (low, carried) = low.overflowing_add(offset);

        if !carried {
            let intermediate = Word { low, high };

            let high = high.wrapping_sub(1);

            (Word { low, high }, OffsetResult::CrossedPage(intermediate))
        } else {
            (Word { low, high }, OffsetResult::SamePage)
        }
    }

    pub fn increment(&self) -> Self {
        let (low, carried) = self.low.overflowing_add(1);

        let high = if carried {
            self.high.wrapping_add(1)
        } else {
            self.high
        };

        Word { high, low }
    }
}

pub enum OffsetResult {
    SamePage,
    CrossedPage(Word),
}
