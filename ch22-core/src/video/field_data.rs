use crate::video::{CRTCRangeType, VideoMemoryAccess, VideoRegisters};

const MAX_LINES: usize = 320;

#[repr(C, packed)]
pub struct Field {
    lines: [FieldLine; MAX_LINES],
}

impl Default for Field {
    fn default() -> Self {
        Field {
            lines: std::array::from_fn(|_| FieldLine::default()),
        }
    }
}

impl Field {
    pub fn clear(&mut self) {
        for line in &mut self.lines {
            line.has_data = false;
        }
    }

    pub fn snapshot_char_data<'a, F>(
        &mut self,
        line_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        ic32_latch: u8,
        video_registers: &VideoRegisters,
        additional_data: FieldLineAdditionalData,
        get_buffer: F,
    ) where
        F: Fn(std::ops::Range<u16>) -> &'a [u8],
    {
        let video_range_type = VideoMemoryAccess::get_crtc_range_type(crtc_address, crtc_length);

        let ula_is_teletext = video_registers.is_teletext();

        let is_line_valid = match (video_range_type, ula_is_teletext) {
            (CRTCRangeType::Teletext, true) => true,
            (CRTCRangeType::HiRes, false) => true,
            _ => false,
        };

        if is_line_valid {
            let (first_ram_range, second_ram_range) =
                VideoMemoryAccess::translate_crtc_range(crtc_address, crtc_length, ic32_latch);

            let first_ram_slice = get_buffer(first_ram_range);
            let second_ram_slice = second_ram_range.map(get_buffer);

            self.lines[line_index].set_data(
                crtc_address,
                video_registers,
                additional_data,
                first_ram_slice,
                second_ram_slice,
            );
        };
    }
}

const MAX_CHARS: usize = 100;
const MAX_BYTES_PER_CHAR: usize = 8;
const MAX_CHAR_DATA: usize = MAX_CHARS * MAX_BYTES_PER_CHAR;

#[repr(C, packed)]
struct FieldLine {
    has_data: bool,
    char_data: [u8; MAX_CHAR_DATA],
    crtc_address: u16,
    video_registers: VideoRegisters,
    additional_data: FieldLineAdditionalData,
}

impl FieldLine {
    fn set_data(
        &mut self,
        crtc_address: u16,
        video_registers: &VideoRegisters,
        additional_data: FieldLineAdditionalData,
        first_slice: &[u8],
        second_slice: Option<&[u8]>,
    ) {
        self.has_data = true;
        self.crtc_address = crtc_address;
        self.video_registers = *video_registers;
        self.additional_data = additional_data;

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
            has_data: false,
            char_data: [0; MAX_CHAR_DATA],
            crtc_address: 0,
            video_registers: VideoRegisters::default(),
            additional_data: FieldLineAdditionalData::default(),
        }
    }
}

#[repr(C, packed)]
#[derive(Default)]
pub struct FieldLineAdditionalData {
    pub d0: u64,
    pub d1: u64,
}
