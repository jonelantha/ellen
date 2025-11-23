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
            line.line_type = FieldLineType::OutOfScan;
        }
    }

    pub fn snapshot_scanline<'a, F>(
        &mut self,
        line_index: usize,
        crtc_memory_address: u16,
        crtc_raster_address_even: u8,
        ic32_latch: u8,
        video_registers: &VideoRegisters,
        get_buffer: F,
    ) where
        F: Fn(std::ops::Range<u16>) -> &'a [u8],
    {
        let crtc_length = video_registers.crtc_registers[1];

        let ula_is_teletext = video_registers.ula_is_teletext();

        if !ula_is_teletext && crtc_raster_address_even as usize >= 8 {
            self.lines[line_index].line_type = FieldLineType::Blank;
            return;
        }

        if !ula_is_teletext && video_registers.crtc_screen_delay_is_no_output() {
            self.lines[line_index].line_type = FieldLineType::Blank;
            return;
        }

        if !is_range_compatible(crtc_memory_address, crtc_length, ula_is_teletext) {
            self.lines[line_index].line_type = FieldLineType::Invalid;
            return;
        }

        let (first_ram_range, second_ram_range) =
            VideoMemoryAccess::translate_crtc_range(crtc_memory_address, crtc_length, ic32_latch);

        let first_ram_slice = get_buffer(first_ram_range);
        let second_ram_slice = second_ram_range.map(get_buffer);

        self.lines[line_index].set_data(
            crtc_memory_address,
            crtc_raster_address_even,
            video_registers,
            first_ram_slice,
            second_ram_slice,
        );
    }
}

fn is_range_compatible(crtc_memory_address: u16, crtc_length: u8, ula_is_teletext: bool) -> bool {
    let video_range_type = VideoMemoryAccess::get_crtc_range_type(crtc_memory_address, crtc_length);

    video_range_type == CRTCRangeType::Teletext && ula_is_teletext
        || video_range_type == CRTCRangeType::HiRes && !ula_is_teletext
}

const MAX_CHARS: usize = 100;

#[repr(C, packed)]
struct FieldLine {
    line_type: FieldLineType,
    char_data: [u8; MAX_CHARS],
    crtc_memory_address: u16,
    crtc_raster_address_even: u8,
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
            line_type: FieldLineType::OutOfScan,
            char_data: [0; MAX_CHARS],
            crtc_memory_address: 0,
            crtc_raster_address_even: 0,
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
        crtc_raster_address_even: u8,
        video_registers: &VideoRegisters,
        first_slice: &[u8],
        second_slice: Option<&[u8]>,
    ) {
        self.line_type = FieldLineType::Visible;
        self.crtc_memory_address = crtc_memory_address;
        self.crtc_raster_address_even = crtc_raster_address_even;

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

        let ula_is_teletext = video_registers.ula_is_teletext();

        let first_end = if ula_is_teletext {
            self.set_char_data(0, first_slice)
        } else {
            self.set_char_data_stride_8(0, first_slice, crtc_raster_address_even)
        };

        if let Some(second_slice) = second_slice {
            if ula_is_teletext {
                self.set_char_data(first_end, second_slice);
            } else {
                self.set_char_data_stride_8(first_end, second_slice, crtc_raster_address_even);
            }
        }
    }

    fn set_char_data(&mut self, dest_start: usize, slice: &[u8]) -> usize {
        let new_length = dest_start + slice.len();

        if new_length > MAX_CHARS {
            panic!("{} > {}", new_length, MAX_CHARS);
        }

        self.char_data[dest_start..new_length].copy_from_slice(slice);

        new_length
    }

    fn set_char_data_stride_8(
        &mut self,
        dest_start: usize,
        slice: &[u8],
        slice_offset: u8,
    ) -> usize {
        if slice.len() % 8 != 0 {
            panic!("{} % 8 != 0", slice.len());
        }

        let new_length = dest_start + slice.len() / 8;

        if new_length > MAX_CHARS {
            panic!("{} > {}", new_length, MAX_CHARS);
        }

        for (i, chunk) in slice.chunks_exact(8).enumerate() {
            self.char_data[dest_start + i] = chunk[slice_offset as usize];
        }

        new_length
    }
}

#[derive(Clone, Copy)]
enum FieldLineType {
    OutOfScan = 0,
    Visible = 1,
    Blank = 2,
    Invalid = 3,
}
