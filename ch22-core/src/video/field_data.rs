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

    pub fn snapshot_scanline<'a, F>(
        &mut self,
        line_index: usize,
        crtc_memory_address: u16,
        crtc_raster_address: u8,
        ic32_latch: u8,
        video_registers: &VideoRegisters,
        get_buffer: F,
    ) where
        F: Fn(std::ops::Range<u16>) -> &'a [u8],
    {
        let crtc_length = video_registers.crtc_registers[1];

        let video_range_type =
            VideoMemoryAccess::get_crtc_range_type(crtc_memory_address, crtc_length);

        let ula_is_teletext = video_registers.ula_is_teletext();

        let is_line_valid = match (video_range_type, ula_is_teletext) {
            (CRTCRangeType::Teletext, true) => true,
            (CRTCRangeType::HiRes, false) => true,
            _ => false,
        };

        if is_line_valid {
            let (first_ram_range, second_ram_range) = VideoMemoryAccess::translate_crtc_range(
                crtc_memory_address,
                crtc_length,
                ic32_latch,
            );

            let first_ram_slice = get_buffer(first_ram_range);
            let second_ram_slice = second_ram_range.map(get_buffer);

            self.lines[line_index].set_data(
                crtc_memory_address,
                crtc_raster_address,
                video_registers,
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
    crtc_memory_address: u16,
    crtc_raster_address: u8,
    crtc_r0_horizontal_total: u8,
    crtc_r1_horizontal_displayed: u8,
    crtc_r2_horizontal_sync_pos: u8,
    crtc_r3_sync_width: u8,
    crtc_r8_interlace_and_delay: u8,
    crtc_r10_cursor_start: u8,
    crtc_r11_cursor_end: u8,
    crtc_r14_cursor_pos_high: u8,
    crtc_r15_cursor_pos_low: u8,
    ula_control: u8,
    ula_palette: u64,
}

impl Default for FieldLine {
    fn default() -> Self {
        FieldLine {
            has_data: false,
            char_data: [0; MAX_CHAR_DATA],
            crtc_memory_address: 0,
            crtc_raster_address: 0,
            crtc_r0_horizontal_total: 0,
            crtc_r1_horizontal_displayed: 0,
            crtc_r2_horizontal_sync_pos: 0,
            crtc_r3_sync_width: 0,
            crtc_r8_interlace_and_delay: 0,
            crtc_r10_cursor_start: 0,
            crtc_r11_cursor_end: 0,
            crtc_r14_cursor_pos_high: 0,
            crtc_r15_cursor_pos_low: 0,
            ula_control: 0,
            ula_palette: 0,
        }
    }
}

impl FieldLine {
    fn set_data(
        &mut self,
        crtc_memory_address: u16,
        crtc_raster_address: u8,
        video_registers: &VideoRegisters,
        first_slice: &[u8],
        second_slice: Option<&[u8]>,
    ) {
        self.has_data = true;
        self.crtc_memory_address = crtc_memory_address;
        self.crtc_raster_address = crtc_raster_address;

        self.ula_control = video_registers.ula_control;
        self.ula_palette = video_registers.ula_palette;

        self.crtc_r0_horizontal_total = video_registers.crtc_registers[0];
        self.crtc_r1_horizontal_displayed = video_registers.crtc_registers[1];
        self.crtc_r2_horizontal_sync_pos = video_registers.crtc_registers[2];
        self.crtc_r3_sync_width = video_registers.crtc_registers[3];
        self.crtc_r8_interlace_and_delay = video_registers.crtc_registers[8];
        self.crtc_r10_cursor_start = video_registers.crtc_registers[10];
        self.crtc_r11_cursor_end = video_registers.crtc_registers[11];
        self.crtc_r14_cursor_pos_high = video_registers.crtc_registers[14];
        self.crtc_r15_cursor_pos_low = video_registers.crtc_registers[15];

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
