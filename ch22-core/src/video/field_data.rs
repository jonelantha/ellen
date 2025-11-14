use crate::video::{CRTCRangeType, VideoMemoryAccess};

const MAX_LINES: usize = 320;

#[repr(C)]
pub struct Field {
    lines: [Option<FieldLine>; MAX_LINES],
}

impl Default for Field {
    fn default() -> Self {
        Field {
            lines: std::array::from_fn(|_| None),
        }
    }
}

impl Field {
    pub fn snapshot_char_data<'a, F>(
        &mut self,
        row_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        ic32_latch: u8,
        required_type: CRTCRangeType,
        get_buffer: F,
    ) where
        F: Fn(std::ops::Range<u16>) -> &'a [u8],
    {
        if crtc_length == 0 {
            self.set_blank_line(row_index);
            return;
        }

        let video_type = VideoMemoryAccess::get_crtc_range_type(crtc_address, crtc_length);

        if video_type != required_type {
            self.set_blank_line(row_index);
            return;
        }

        let (first_ram_range, second_ram_range) =
            VideoMemoryAccess::translate_crtc_range(crtc_address, crtc_length, ic32_latch);

        let first_ram_slice = get_buffer(first_ram_range);
        let second_ram_slice = second_ram_range.map(|range| get_buffer(range));

        self.set_char_data_line(row_index, first_ram_slice, second_ram_slice);
    }

    pub fn set_blank_line(&mut self, row_index: usize) {
        self.lines[row_index] = None;
    }

    pub fn set_char_data_line(
        &mut self,
        row_index: usize,
        first_slice: &[u8],
        second_slice: Option<&[u8]>,
    ) {
        let row = self.lines[row_index].get_or_insert_default();

        row.set_char_data(first_slice, second_slice);
    }
}

const MAX_CHARS: usize = 100;
const MAX_BYTES_PER_CHAR: usize = 8;
const MAX_CHAR_DATA: usize = MAX_CHARS * MAX_BYTES_PER_CHAR;

#[repr(C)]
struct FieldLine {
    char_data: [u8; MAX_CHAR_DATA],
}

impl FieldLine {
    fn set_char_data(&mut self, first_slice: &[u8], second_slice: Option<&[u8]>) {
        let first_length = self.set_char_data_chunk(0, first_slice);

        if let Some(second_slice) = second_slice {
            self.set_char_data_chunk(first_length, second_slice);
        }
    }

    fn set_char_data_chunk(&mut self, start: usize, chunk: &[u8]) -> usize {
        let new_length = start + chunk.len();

        if new_length > MAX_CHAR_DATA {
            panic!("{} > {}", new_length, MAX_CHAR_DATA);
        }

        self.char_data[start..new_length].copy_from_slice(chunk);

        new_length
    }
}

impl Default for FieldLine {
    fn default() -> Self {
        FieldLine {
            char_data: [0; MAX_CHAR_DATA],
        }
    }
}
