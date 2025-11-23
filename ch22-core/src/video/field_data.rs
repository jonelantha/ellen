use crate::video::{CRTCRangeType, FieldLine, VideoMemoryAccess, VideoRegisters};

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
        let crtc_length = video_registers.crtc_r1_horizontal_displayed;

        self.lines[line_index].set_registers(
            crtc_memory_address,
            crtc_raster_address_even,
            video_registers,
        );

        let ula_is_teletext = video_registers.ula_is_teletext();

        if !ula_is_teletext && crtc_raster_address_even as usize >= 8 {
            self.lines[line_index].set_blank();
            return;
        }

        if !ula_is_teletext && video_registers.crtc_screen_delay_is_no_output() {
            self.lines[line_index].set_blank();
            return;
        }

        if !is_range_compatible(crtc_memory_address, crtc_length, ula_is_teletext) {
            self.lines[line_index].set_invalid();
            return;
        }

        let (first_ram_range, second_ram_range) =
            VideoMemoryAccess::translate_crtc_range(crtc_memory_address, crtc_length, ic32_latch);

        let first_ram_slice = get_buffer(first_ram_range);
        let second_ram_slice = second_ram_range.map(get_buffer);

        if ula_is_teletext {
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

fn is_range_compatible(crtc_memory_address: u16, crtc_length: u8, ula_is_teletext: bool) -> bool {
    let video_range_type = VideoMemoryAccess::get_crtc_range_type(crtc_memory_address, crtc_length);

    video_range_type == CRTCRangeType::Teletext && ula_is_teletext
        || video_range_type == CRTCRangeType::HiRes && !ula_is_teletext
}
