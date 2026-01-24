const MAX_CHARS: usize = 100;

#[repr(C, packed)]
pub struct FieldLine {
    flags: u8,
    pub(crate) ula_control: u8,
    pub(crate) total_chars: u8,
    pub(crate) back_porch: u8,
    pub(crate) cursor_char: u8,
    pad: [u8; 3],
    pub(crate) ula_palette: u64,
    pub(crate) char_data: [u8; MAX_CHARS],
}

impl Default for FieldLine {
    fn default() -> Self {
        FieldLine {
            flags: 0,
            ula_control: 0,
            total_chars: 0,
            back_porch: 0,
            cursor_char: 0,
            pad: [0; 3],
            ula_palette: 0,
            char_data: [0; MAX_CHARS],
        }
    }
}

impl FieldLine {
    pub fn clear(&mut self) {
        self.flags = 0;
    }

    pub fn set_displayed(&mut self) {
        self.flags |= flags::DISPLAYED;
    }

    pub fn set_invalid_range(&mut self) {
        self.flags |= flags::INVALID_RANGE;
    }

    pub fn set_interlace_video_and_sync(&mut self) {
        self.flags |= flags::INTERLACE_VIDEO_AND_SYNC;
    }

    pub fn set_cursor_raster_flags(&mut self, even: bool, odd: bool) {
        self.flags &= !(flags::CURSOR_RASTER_EVEN | flags::CURSOR_RASTER_ODD);

        if even {
            self.flags |= flags::CURSOR_RASTER_EVEN;
        }
        if odd {
            self.flags |= flags::CURSOR_RASTER_ODD;
        }
    }

    pub fn set_char_data(&mut self, first_slice: &[u8], second_slice: Option<&[u8]>) {
        self.flags |= flags::HAS_BYTES;
        let first_end = first_slice.len();

        debug_assert!(first_end <= MAX_CHARS);

        self.char_data[..first_end].copy_from_slice(first_slice);

        if let Some(second_slice) = second_slice {
            let second_end = first_end + second_slice.len();

            debug_assert!(second_end <= MAX_CHARS);

            self.char_data[first_end..second_end].copy_from_slice(second_slice);
        }
    }

    pub fn set_char_data_for_raster(
        &mut self,
        first_slice: &[u8],
        second_slice: Option<&[u8]>,
        raster_line: u8,
    ) {
        self.flags |= flags::HAS_BYTES;

        let first_end = copy_into_stride_8(&mut self.char_data, 0, first_slice, raster_line);

        if let Some(second_slice) = second_slice {
            copy_into_stride_8(&mut self.char_data, first_end, second_slice, raster_line);
        }
    }

    // Test-only method to get raw data of line in memory (available only for tests)
    #[cfg(test)]
    pub fn get_raw_data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const FieldLine) as *const u8,
                std::mem::size_of::<FieldLine>(),
            )
        }
    }
}

fn copy_into_stride_8(
    dest: &mut [u8],
    dest_start: usize,
    source: &[u8],
    source_offset: u8,
) -> usize {
    debug_assert!(source.len() % 8 == 0);

    let new_length = dest_start + (source.len() >> 3);

    debug_assert!(new_length <= MAX_CHARS);

    for (i, chunk) in source.chunks_exact(8).enumerate() {
        dest[dest_start + i] = chunk[source_offset as usize];
    }

    new_length
}

pub mod flags {
    pub const DISPLAYED: u8 = 0b0000_0001;
    pub const HAS_BYTES: u8 = 0b0000_0010;
    pub const INVALID_RANGE: u8 = 0b0000_0100;
    pub const INTERLACE_VIDEO_AND_SYNC: u8 = 0b0000_1000;
    pub const CURSOR_RASTER_EVEN: u8 = 0b0001_0000;
    pub const CURSOR_RASTER_ODD: u8 = 0b0010_0000;
}
