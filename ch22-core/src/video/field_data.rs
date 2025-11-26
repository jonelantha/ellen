use crate::video::{FieldLine, VideoMemoryAccess, VideoRegisters};

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
        self.lines[line_index].set_registers(
            crtc_memory_address,
            crtc_raster_address_even,
            video_registers,
        );

        let crtc_length = video_registers.crtc_r1_horizontal_displayed;
        let ula_teletext = video_registers.is_ula_teletext();

        if crtc_length == 0
            || (!ula_teletext && crtc_raster_address_even >= 8)
            || (!ula_teletext && video_registers.is_crtc_screen_delay_no_output())
        {
            self.lines[line_index].set_blank();
            return;
        }

        if (ula_teletext
            && !VideoMemoryAccess::is_crtc_range_telextext(crtc_memory_address, crtc_length))
            || (!ula_teletext
                && !VideoMemoryAccess::is_crtc_range_hires(crtc_memory_address, crtc_length))
        {
            self.lines[line_index].set_invalid();
            return;
        }

        let (first_ram_range, second_ram_range) =
            VideoMemoryAccess::translate_crtc_range(crtc_memory_address, crtc_length, ic32_latch);

        let first_ram_slice = get_buffer(first_ram_range);
        let second_ram_slice = second_ram_range.map(get_buffer);

        if ula_teletext {
            self.lines[line_index].set_char_data(first_ram_slice, second_ram_slice);
        } else {
            self.lines[line_index].set_char_data_for_raster(
                first_ram_slice,
                second_ram_slice,
                crtc_raster_address_even,
            );
        }
    }
}
