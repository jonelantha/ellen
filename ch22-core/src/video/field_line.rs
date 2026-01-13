use crate::video::VideoRegisters;

const MAX_CHARS: usize = 100;

#[repr(C, packed)]
pub struct FieldLine {
    flags: u8,
    char_data: [u8; MAX_CHARS],
    crtc_memory_address: u16,
    crtc_r0_horizontal_total: u8,
    crtc_r1_horizontal_displayed: u8,
    crtc_r2_horizontal_sync_position: u8,
    crtc_r3_sync_width: u8,
    crtc_r8_interlace_and_skew: u8,
    crtc_r14_cursor_h: u8,
    crtc_r15_cursor_l: u8,
    ula_control: u8,
    ula_palette: u64,
}

impl Default for FieldLine {
    fn default() -> Self {
        FieldLine {
            flags: 0,
            char_data: [0; MAX_CHARS],
            crtc_memory_address: 0,
            crtc_r0_horizontal_total: 0,
            crtc_r1_horizontal_displayed: 0,
            crtc_r2_horizontal_sync_position: 0,
            crtc_r3_sync_width: 0,
            crtc_r8_interlace_and_skew: 0,
            crtc_r14_cursor_h: 0,
            crtc_r15_cursor_l: 0,
            ula_control: 0,
            ula_palette: 0,
        }
    }
}

impl FieldLine {
    pub fn clear(&mut self) {
        self.flags = 0;
    }

    pub fn set_registers(&mut self, crtc_memory_address: u16, video_registers: &VideoRegisters) {
        self.crtc_memory_address = crtc_memory_address;

        self.ula_control = video_registers.ula_control;
        self.ula_palette = video_registers.ula_palette;

        self.crtc_r0_horizontal_total = video_registers.crtc_r0_horizontal_total;
        self.crtc_r1_horizontal_displayed = video_registers.crtc_r1_horizontal_displayed;
        self.crtc_r2_horizontal_sync_position = video_registers.crtc_r2_horizontal_sync_position;
        self.crtc_r3_sync_width = video_registers.crtc_r3_sync_width;
        self.crtc_r8_interlace_and_skew = video_registers.crtc_r8_interlace_and_skew;
        self.crtc_r14_cursor_h = video_registers.crtc_r14_cursor_h;
        self.crtc_r15_cursor_l = video_registers.crtc_r15_cursor_l;
    }

    pub fn set_cursor_flags(
        &mut self,
        crtc_raster_address_even: u8,
        crtc_raster_address_odd: u8,
        field_counter: u8,
        video_registers: &VideoRegisters,
    ) {
        if !is_cursor_blink_visible(video_registers.crtc_r10_cursor_start_raster, field_counter) {
            return;
        }

        if is_cursor_raster(
            video_registers.crtc_r10_cursor_start_raster,
            video_registers.crtc_r11_cursor_end_raster,
            crtc_raster_address_even,
        ) {
            self.flags |= flags::CURSOR_RASTER_EVEN;
        }

        if is_cursor_raster(
            video_registers.crtc_r10_cursor_start_raster,
            video_registers.crtc_r11_cursor_end_raster,
            crtc_raster_address_odd,
        ) {
            self.flags |= flags::CURSOR_RASTER_ODD;
        }
    }

    pub fn set_displayed(&mut self) {
        self.flags |= flags::DISPLAYED;
    }

    pub fn set_invalid_range(&mut self) {
        self.flags |= flags::INVALID_RANGE;
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
    pub const CURSOR_RASTER_EVEN: u8 = 0b0001_0000;
    pub const CURSOR_RASTER_ODD: u8 = 0b0010_0000;
}

fn is_cursor_blink_visible(crtc_r10_cursor_start_raster: u8, field_counter: u8) -> bool {
    let blink_mode = crtc_r10_cursor_start_raster & 0b0110_0000;

    match blink_mode {
        0b0000_0000 => true,                      // Always on
        0b0010_0000 => false,                     // Hidden
        0b0100_0000 => field_counter & 0x08 != 0, // Fast blink
        0b0110_0000 => field_counter & 0x10 != 0, // Slow blink
        _ => unreachable!(),
    }
}

fn is_cursor_raster(
    crtc_r10_cursor_start_raster: u8,
    crtc_r11_cursor_end_raster: u8,
    crtc_raster_address: u8,
) -> bool {
    let cursor_start = crtc_r10_cursor_start_raster & 0x1f;
    let cursor_end = crtc_r11_cursor_end_raster;

    crtc_raster_address >= cursor_start && crtc_raster_address <= cursor_end
}
