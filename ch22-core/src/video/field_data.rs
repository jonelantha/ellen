use std::cmp::{max, min};

use crate::video::{
    FieldLine, VideoMemoryAccess, VideoRegisters,
    video_registers::{R8_CURSOR_DELAY_HIDDEN, R10CursorBlinkMode},
};

#[cfg(test)]
mod tests;

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
            line.clear();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn snapshot_scanline<'a>(
        &mut self,
        line_index: usize,
        crtc_memory_address: u16,
        crtc_raster_address_even: u8,
        crtc_raster_address_odd: u8,
        ic32_latch: u8,
        field_counter: u8,
        video_registers: &VideoRegisters,
        get_buffer: impl Fn(std::ops::Range<u16>) -> &'a [u8],
    ) {
        let line = &mut self.lines[line_index];

        set_line_metrics(line, video_registers);

        set_line_ula_fields(line, video_registers);

        set_line_cursor_fields(
            line,
            crtc_raster_address_even,
            crtc_raster_address_odd,
            field_counter,
            crtc_memory_address,
            video_registers,
        );

        if video_registers.ula_is_teletext() {
            snapshot_teletext_scanline_data(line, crtc_memory_address, video_registers, get_buffer);
        } else {
            snapshot_hires_scanline_raster_data(
                line,
                crtc_memory_address,
                crtc_raster_address_even,
                ic32_latch,
                video_registers,
                get_buffer,
            );
        }
    }
}

fn snapshot_teletext_scanline_data<'a>(
    field_line: &mut FieldLine,
    crtc_memory_address: u16,
    video_registers: &VideoRegisters,
    get_buffer: impl Fn(std::ops::Range<u16>) -> &'a [u8],
) {
    let crtc_length = video_registers.crtc_r1_horizontal_displayed;

    if crtc_length == 0 {
        return;
    }

    match VideoMemoryAccess::translate_crtc_teletext_range(crtc_memory_address, crtc_length) {
        None => field_line.set_invalid_range(),
        Some(ranges) => field_line.set_char_data(get_buffer(ranges.0), ranges.1.map(get_buffer)),
    }
}

fn snapshot_hires_scanline_raster_data<'a>(
    field_line: &mut FieldLine,
    crtc_memory_address: u16,
    crtc_raster_address_even: u8,
    ic32_latch: u8,
    video_registers: &VideoRegisters,
    get_buffer: impl Fn(std::ops::Range<u16>) -> &'a [u8],
) {
    let crtc_length = video_registers.crtc_r1_horizontal_displayed;

    if crtc_length == 0
        || crtc_raster_address_even >= 8
        || video_registers.r8_is_crtc_screen_delay_no_output()
    {
        return;
    }

    match VideoMemoryAccess::translate_crtc_hires_range(
        crtc_memory_address,
        crtc_length,
        ic32_latch,
    ) {
        None => field_line.set_invalid_range(),
        Some(ranges) => field_line.set_char_data_for_raster(
            get_buffer(ranges.0),
            ranges.1.map(get_buffer),
            crtc_raster_address_even,
        ),
    }
}

fn set_line_metrics(field_line: &mut FieldLine, video_registers: &VideoRegisters) {
    field_line.set_displayed();

    field_line.total_chars = video_registers.crtc_r1_horizontal_displayed;
    field_line.back_porch = calc_back_porch(video_registers);

    if video_registers.r8_is_interlace_sync_and_video() {
        field_line.set_interlace_video_and_sync();
    }
}

fn set_line_ula_fields(field_line: &mut FieldLine, video_registers: &VideoRegisters) {
    field_line.ula_control = video_registers.ula_control;
    field_line.ula_palette = video_registers.ula_palette;
}

fn set_line_cursor_fields(
    field_line: &mut FieldLine,
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

    field_line.cursor_char = r8_cursor_delay + rel_address as u8;
    field_line.set_cursor_raster_flags(is_even_in_range, is_odd_in_range);
}

fn is_r10_cursor_blink_visible(
    r10_cursor_blink_mode: R10CursorBlinkMode,
    field_counter: u8,
) -> bool {
    match r10_cursor_blink_mode {
        R10CursorBlinkMode::Solid => true,
        R10CursorBlinkMode::Hidden => false,
        R10CursorBlinkMode::Fast => field_counter & 0x08 != 0,
        R10CursorBlinkMode::Slow => field_counter & 0x10 != 0,
    }
}

fn calc_back_porch(video_registers: &VideoRegisters) -> u8 {
    let full_horizontal_total = (video_registers.crtc_r0_horizontal_total as u16) + 1;

    let h_sync_width = max(video_registers.r3_h_sync_width(), 1);

    let h_sync_end = min(
        video_registers.crtc_r2_horizontal_sync_position as u16 + h_sync_width as u16,
        full_horizontal_total,
    );

    (full_horizontal_total - h_sync_end) as u8
}
