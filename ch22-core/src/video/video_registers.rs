use std::ops::RangeInclusive;

#[derive(Default, Copy, Clone)]
#[repr(C, packed)]
pub struct VideoRegisters {
    pub ula_control: u8,
    pub ula_palette: u64,

    pub crtc_r0_horizontal_total: u8,
    pub crtc_r1_horizontal_displayed: u8,
    pub crtc_r2_horizontal_sync_position: u8,
    pub crtc_r3_sync_width: u8,
    pub crtc_r4_vertical_total: u8,
    pub crtc_r5_vertical_total_adjust: u8,
    pub crtc_r6_vertical_displayed: u8,
    pub crtc_r7_vertical_sync_position: u8,
    pub crtc_r8_interlace_and_skew: u8,
    pub crtc_r9_maximum_raster_address: u8,
    pub crtc_r10_cursor_start_raster: u8,
    pub crtc_r11_cursor_end_raster: u8,
    pub crtc_r12_start_address_h: u8,
    pub crtc_r13_start_address_l: u8,
    pub crtc_r14_cursor_h: u8,
    pub crtc_r15_cursor_l: u8,
}

impl VideoRegisters {
    pub fn reset(&mut self) {
        self.ula_control = 0x9c;

        self.crtc_r0_horizontal_total = 0x7f;
        self.crtc_r1_horizontal_displayed = 0x50;
        self.crtc_r2_horizontal_sync_position = 0x62;
        self.crtc_r3_sync_width = 0x28;
        self.crtc_r4_vertical_total = 0x26;
        self.crtc_r5_vertical_total_adjust = 0x00;
        self.crtc_r6_vertical_displayed = 0x20;
        self.crtc_r7_vertical_sync_position = 0x22;
        self.crtc_r8_interlace_and_skew = 0x00;
        self.crtc_r9_maximum_raster_address = 0x07;
        self.crtc_r10_cursor_start_raster = 0x00;
        self.crtc_r11_cursor_end_raster = 0x00;
        self.crtc_r12_start_address_h = 0x06;
        self.crtc_r13_start_address_l = 0x00;
        self.crtc_r14_cursor_h = 0x00;
        self.crtc_r15_cursor_l = 0x00;
    }

    pub fn set_ula_palette(&mut self, entry: u8, value: u8) {
        let shift = entry * 4;

        self.ula_palette &= !(0x0f << shift);
        self.ula_palette |= ((value & 0x0f) as u64) << shift;
    }

    pub fn ula_is_teletext(&self) -> bool {
        (self.ula_control & 0x02) != 0
    }

    pub fn r8_is_crtc_screen_delay_no_output(&self) -> bool {
        self.crtc_r8_interlace_and_skew & 0x30 == 0x30
    }

    pub fn r8_is_interlace_sync_and_video(&self) -> bool {
        self.crtc_r8_interlace_and_skew & 0x03 == 0x03
    }

    pub fn r10_cursor_blink_mode(&self) -> R10CursorBlinkMode {
        match self.crtc_r10_cursor_start_raster & 0x60 {
            0x00 => R10CursorBlinkMode::Solid,
            0x20 => R10CursorBlinkMode::Hidden,
            0x40 => R10CursorBlinkMode::Fast,
            0x60 => R10CursorBlinkMode::Slow,
            _ => unreachable!(),
        }
    }

    pub fn r10_r11_cursor_raster_range(&self) -> RangeInclusive<u8> {
        (self.crtc_r10_cursor_start_raster & 0x1f)..=self.crtc_r11_cursor_end_raster
    }

    pub fn r8_cursor_delay(&self) -> u8 {
        (self.crtc_r8_interlace_and_skew & 0xc0) >> 6
    }

    pub fn r14_r15_cursor_address(&self) -> u16 {
        (self.crtc_r14_cursor_h as u16) << 8 | self.crtc_r15_cursor_l as u16
    }

    #[cfg(test)]
    pub fn get_crtc_register(&self, control_reg: u8) -> u8 {
        match control_reg {
            0 => self.crtc_r0_horizontal_total,
            1 => self.crtc_r1_horizontal_displayed,
            2 => self.crtc_r2_horizontal_sync_position,
            3 => self.crtc_r3_sync_width,
            4 => self.crtc_r4_vertical_total,
            5 => self.crtc_r5_vertical_total_adjust,
            6 => self.crtc_r6_vertical_displayed,
            7 => self.crtc_r7_vertical_sync_position,
            8 => self.crtc_r8_interlace_and_skew,
            9 => self.crtc_r9_maximum_raster_address,
            10 => self.crtc_r10_cursor_start_raster,
            11 => self.crtc_r11_cursor_end_raster,
            12 => self.crtc_r12_start_address_h,
            13 => self.crtc_r13_start_address_l,
            14 => self.crtc_r14_cursor_h,
            15 => self.crtc_r15_cursor_l,
            _ => panic!("Invalid control_reg {}", control_reg),
        }
    }
}

pub const R8_CURSOR_DELAY_HIDDEN: u8 = 3;

pub enum R10CursorBlinkMode {
    Solid,
    Hidden,
    Fast,
    Slow,
}
