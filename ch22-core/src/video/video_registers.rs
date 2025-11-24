#[derive(Default, Copy, Clone)]
#[repr(C, packed)]
pub struct VideoRegisters {
    pub ula_control: u8,
    pub ula_palette: u64,

    pub crtc_r0_horizontal_total: u8,
    pub crtc_r1_horizontal_displayed: u8,
    pub crtc_r2_horizontal_sync_pos: u8,
    pub crtc_r3_sync_width: u8,
    pub crtc_r4_vertical_total: u8,
    pub crtc_r5_vertical_total_adjust: u8,
    pub crtc_r6_vertical_displayed: u8,
    pub crtc_r7_vertical_sync_pos: u8,
    pub crtc_r8_interlace_and_skew: u8,
    pub crtc_r9_maximum_raster_address: u8,
    pub crtc_r10_cursor_start_raster: u8,
    pub crtc_r11_cursor_end_raster: u8,
    pub crtc_r12_start_address_high: u8,
    pub crtc_r13_start_address_low: u8,
    pub crtc_r14_cursor_high: u8,
    pub crtc_r15_cursor_low: u8,
}

impl VideoRegisters {
    pub fn reset(&mut self) {
        self.ula_control = 0x9c;

        self.crtc_r0_horizontal_total = 0x7f;
        self.crtc_r1_horizontal_displayed = 0x50;
        self.crtc_r2_horizontal_sync_pos = 0x62;
        self.crtc_r3_sync_width = 0x28;
        self.crtc_r4_vertical_total = 0x26;
        self.crtc_r5_vertical_total_adjust = 0x00;
        self.crtc_r6_vertical_displayed = 0x20;
        self.crtc_r7_vertical_sync_pos = 0x22;
        self.crtc_r8_interlace_and_skew = 0x00;
        self.crtc_r9_maximum_raster_address = 0x07;
        self.crtc_r10_cursor_start_raster = 0x00;
        self.crtc_r11_cursor_end_raster = 0x00;
        self.crtc_r12_start_address_high = 0x06;
        self.crtc_r13_start_address_low = 0x00;
        self.crtc_r14_cursor_high = 0x00;
        self.crtc_r15_cursor_low = 0x00;
    }

    pub fn set_ula_palette(&mut self, entry: u8, value: u8) {
        let shift = entry * 4;

        self.ula_palette &= !(0x0f << shift);
        self.ula_palette |= ((value & 0x0f) as u64) << shift;
    }

    pub fn is_ula_teletext(&self) -> bool {
        (self.ula_control & 0x02) != 0
    }

    pub fn is_crtc_screen_delay_no_output(&self) -> bool {
        self.crtc_r8_interlace_and_skew & 0x30 == 0x30
    }
}
