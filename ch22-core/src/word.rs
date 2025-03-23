/// low, high
#[derive(Clone, Copy, Default)]
pub struct Word(pub u8, pub u8);

impl From<Word> for u16 {
    fn from(Word(low, high): Word) -> Self {
        u16::from_le_bytes([low, high])
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        let [low, high] = u16::to_le_bytes(value);

        Word(low, high)
    }
}

impl Word {
    pub fn zero_page(zero_page_address: u8) -> Self {
        Word(zero_page_address, 0)
    }

    pub fn stack_page(stack_pointer: u8) -> Self {
        Word(stack_pointer, 1)
    }

    pub fn same_page_add(&self, offset: u8) -> Self {
        let Word(low, high) = *self;

        Word(low.wrapping_add(offset), high)
    }

    pub fn paged_add(&self, offset: u8) -> (Word, OffsetResult) {
        let Word(low, high) = *self;

        let (low, carried) = low.overflowing_add(offset);

        if carried {
            let intermediate = Word(low, high);

            let high = high.wrapping_add(1);

            (Word(low, high), OffsetResult::CrossedPage(intermediate))
        } else {
            (Word(low, high), OffsetResult::SamePage)
        }
    }

    pub fn paged_subtract(&self, offset: u8) -> (Word, OffsetResult) {
        let Word(low, high) = *self;

        let (low, carried) = low.overflowing_add(offset);

        if !carried {
            let intermediate = Word(low, high);

            let high = high.wrapping_sub(1);

            (Word(low, high), OffsetResult::CrossedPage(intermediate))
        } else {
            (Word(low, high), OffsetResult::SamePage)
        }
    }

    pub fn increment(&mut self) {
        let carried;
        (self.0, carried) = self.0.overflowing_add(1);

        if carried {
            self.1 = self.1.wrapping_add(1);
        }
    }
}

pub enum OffsetResult {
    SamePage,
    CrossedPage(Word),
}
