use crate::video::VideoRegisters;

const MAX_CHARS: usize = 100;

#[repr(C, packed)]
pub struct FieldLine {
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
    pub fn set_registers(
        &mut self,
        crtc_memory_address: u16,
        crtc_raster_address_even: u8,
        video_registers: &VideoRegisters,
    ) {
        self.crtc_memory_address = crtc_memory_address;
        self.crtc_raster_address_even = crtc_raster_address_even;

        self.ula_control = video_registers.ula_control;
        self.ula_palette = video_registers.ula_palette;

        self.crtc_r0_horizontal_total = video_registers.crtc_r0_horizontal_total;
        self.crtc_r1_horizontal_displayed = video_registers.crtc_r1_horizontal_displayed;
        self.crtc_r2_horizontal_sync_pos = video_registers.crtc_r2_horizontal_sync_pos;
        self.crtc_r3_sync_width = video_registers.crtc_r3_sync_width;
        self.crtc_r8_interlace_and_delay = video_registers.crtc_r8_interlace_and_delay;
        self.crtc_r10_cursor_start = video_registers.crtc_r10_cursor_start;
        self.crtc_r11_cursor_end = video_registers.crtc_r11_cursor_end;
        self.crtc_r14_cursor_pos_high = video_registers.crtc_r14_cursor_pos_high;
        self.crtc_r15_cursor_pos_low = video_registers.crtc_r15_cursor_pos_low;
    }

    pub fn set_out_of_scan(&mut self) {
        self.line_type = FieldLineType::OutOfScan;
    }

    pub fn set_invalid(&mut self) {
        self.line_type = FieldLineType::Invalid;
    }

    pub fn set_blank(&mut self) {
        self.line_type = FieldLineType::Blank;
    }

    pub fn set_char_data(&mut self, first_slice: &[u8], second_slice: Option<&[u8]>) {
        self.line_type = FieldLineType::Visible;

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
        self.line_type = FieldLineType::Visible;

        let first_end = copy_into_stride_8(&mut self.char_data, 0, first_slice, raster_line);

        if let Some(second_slice) = second_slice {
            copy_into_stride_8(&mut self.char_data, first_end, second_slice, raster_line);
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

#[derive(Clone, Copy)]
enum FieldLineType {
    OutOfScan = 0,
    Visible = 1,
    Blank = 2,
    Invalid = 3,
}
