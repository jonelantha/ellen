const MAX_LINES: usize = 320;
pub const MODE7_CHARS_PER_LINE: usize = 80;

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
