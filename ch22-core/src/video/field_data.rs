use crate::video::{FieldLine, VideoMemoryAccess, VideoRegisters};

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
            line.set_out_of_scan();
        }
    }

    pub fn snapshot_scanline<'a>(
        &mut self,
        line_index: usize,
        crtc_memory_address: u16,
        crtc_raster_address_even: u8,
        ic32_latch: u8,
        video_registers: &VideoRegisters,
        get_buffer: impl Fn(std::ops::Range<u16>) -> &'a [u8],
    ) {
        self.lines[line_index].set_registers(
            crtc_memory_address,
            crtc_raster_address_even,
            video_registers,
        );

        if video_registers.is_ula_teletext() {
            snapshot_teletext_scanline_data(
                &mut self.lines[line_index],
                crtc_memory_address,
                ic32_latch,
                video_registers,
                get_buffer,
            );
        } else {
            snapshot_hires_scanline_raster_data(
                &mut self.lines[line_index],
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
    ic32_latch: u8,
    video_registers: &VideoRegisters,
    get_buffer: impl Fn(std::ops::Range<u16>) -> &'a [u8],
) {
    let crtc_length = video_registers.crtc_r1_horizontal_displayed;
    if crtc_length == 0 {
        field_line.set_blank();
        return;
    }

    if !VideoMemoryAccess::is_crtc_range_telextext(crtc_memory_address, crtc_length) {
        field_line.set_invalid();
        return;
    }

    let (first_ram_range, second_ram_range) =
        VideoMemoryAccess::translate_crtc_range(crtc_memory_address, crtc_length, ic32_latch);

    field_line.set_char_data(
        get_buffer(first_ram_range),
        second_ram_range.map(get_buffer),
    );
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
        || video_registers.is_crtc_screen_delay_no_output()
    {
        field_line.set_blank();
        return;
    }

    if !VideoMemoryAccess::is_crtc_range_hires(crtc_memory_address, crtc_length) {
        field_line.set_invalid();
        return;
    }

    let (first_ram_range, second_ram_range) =
        VideoMemoryAccess::translate_crtc_range(crtc_memory_address, crtc_length, ic32_latch);

    field_line.set_char_data_for_raster(
        get_buffer(first_ram_range),
        second_ram_range.map(get_buffer),
        crtc_raster_address_even,
    );
}
