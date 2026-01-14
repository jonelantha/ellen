use std::cmp::{max, min};

use crate::video::{
    VideoRegisters,
    video_registers::{R8_CURSOR_DELAY_HIDDEN, R10CursorBlinkMode},
};

const MAX_CHARS: usize = 100;

#[repr(C, packed)]
pub struct FieldLine {
    flags: u8,
    char_data: [u8; MAX_CHARS],
    crtc_r1_horizontal_displayed: u8,
    back_porch: u8,
    ula_control: u8,
    ula_palette: u64,
    cursor_char: u8,
}

impl Default for FieldLine {
    fn default() -> Self {
        FieldLine {
            flags: 0,
            char_data: [0; MAX_CHARS],
            crtc_r1_horizontal_displayed: 0,
            back_porch: 0,
            ula_control: 0,
            ula_palette: 0,
            cursor_char: 0,
        }
    }
}

impl FieldLine {
    pub fn clear(&mut self) {
        self.flags = 0;
    }

    pub fn set_registers(&mut self, video_registers: &VideoRegisters) {
        self.ula_control = video_registers.ula_control;
        self.ula_palette = video_registers.ula_palette;

        self.crtc_r1_horizontal_displayed = video_registers.crtc_r1_horizontal_displayed;
    }

    pub fn set_cursor(
        &mut self,
        crtc_raster_address_even: u8,
        crtc_raster_address_odd: u8,
        field_counter: u8,
        crtc_memory_address: u16,
        video_registers: &VideoRegisters,
    ) {
        let r10_r11_cursor_raster_range = video_registers.r10_r11_cursor_raster_range();

        let is_even_in_range = r10_r11_cursor_raster_range.contains(&crtc_raster_address_even);

        let is_odd_in_range = r10_r11_cursor_raster_range.contains(&crtc_raster_address_odd);

        if !is_even_in_range && !is_odd_in_range {
            return;
        }

        if !is_r10_cursor_blink_visible(video_registers.r10_cursor_blink_mode(), field_counter) {
            return;
        }

        let r8_cursor_delay = video_registers.r8_cursor_delay();
        if r8_cursor_delay == R8_CURSOR_DELAY_HIDDEN {
            return;
        }

        let r14_r15_cursor_address = video_registers.r14_r15_cursor_address();

        if r14_r15_cursor_address < crtc_memory_address {
            return;
        }

        let rel_address = r14_r15_cursor_address - crtc_memory_address;
        if rel_address >= video_registers.crtc_r1_horizontal_displayed as u16 {
            return;
        }

        self.cursor_char = r8_cursor_delay + rel_address as u8;

        if is_even_in_range {
            self.flags |= flags::CURSOR_RASTER_EVEN;
        }

        if is_odd_in_range {
            self.flags |= flags::CURSOR_RASTER_ODD;
        }
    }

    pub fn set_back_porch(&mut self, video_registers: &VideoRegisters) {
        let full_horizontal_total = (video_registers.crtc_r0_horizontal_total as u16) + 1;

        let h_sync_width = max(video_registers.r3_h_sync_width(), 1);

        let h_sync_end = min(
            video_registers.crtc_r2_horizontal_sync_position as u16 + h_sync_width as u16,
            full_horizontal_total as u16,
        );

        self.back_porch = (full_horizontal_total - h_sync_end) as u8;
    }

    pub fn update_interlace_video_and_sync(&mut self, video_registers: &VideoRegisters) {
        if video_registers.r8_is_interlace_sync_and_video() {
            self.flags |= flags::INTERLACE_VIDEO_AND_SYNC;
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
    pub const INTERLACE_VIDEO_AND_SYNC: u8 = 0b0000_1000;
    pub const CURSOR_RASTER_EVEN: u8 = 0b0001_0000;
    pub const CURSOR_RASTER_ODD: u8 = 0b0010_0000;
}

fn is_r10_cursor_blink_visible(
    r10_cursor_blink_mode: R10CursorBlinkMode,
    field_counter: u8,
) -> bool {
    match r10_cursor_blink_mode {
        R10CursorBlinkMode::Solid => true,                     // Always on
        R10CursorBlinkMode::Hidden => false,                   // Hidden
        R10CursorBlinkMode::Fast => field_counter & 0x08 != 0, // Fast blink
        R10CursorBlinkMode::Slow => field_counter & 0x10 != 0, // Slow blink
    }
}
