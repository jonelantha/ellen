#[derive(Default, Copy, Clone)]
#[repr(C, packed)]
pub struct VideoRegisters {
    pub ula_control: u8,
    pub ula_palette: u64,
    pub crtc_registers: [u8; 18],
}

impl VideoRegisters {
    pub fn reset(&mut self) {
        self.ula_control = 0x9c;

        self.crtc_registers[0] = 0x7f;
        self.crtc_registers[1] = 0x50;
        self.crtc_registers[2] = 0x62;
        self.crtc_registers[3] = 0x28;
        self.crtc_registers[4] = 0x26;
        self.crtc_registers[5] = 0x00;
        self.crtc_registers[6] = 0x20;
        self.crtc_registers[7] = 0x22;
        self.crtc_registers[8] = 0x00;
        self.crtc_registers[9] = 0x07;
        self.crtc_registers[10] = 0x00;
        self.crtc_registers[11] = 0x00;
        self.crtc_registers[12] = 0x06;
        self.crtc_registers[13] = 0x00;
        self.crtc_registers[14] = 0x00;
        self.crtc_registers[15] = 0x00;
    }

    pub fn set_ula_palette(&mut self, entry: u8, value: u8) {
        let shift = entry * 4;

        self.ula_palette &= !(0x0f << shift);
        self.ula_palette |= ((value & 0x0f) as u64) << shift;
    }

    pub fn ula_is_teletext(&self) -> bool {
        (self.ula_control & 0x02) != 0
    }

    pub fn crtc_screen_delay_is_no_output(&self) -> bool {
        self.crtc_registers[8] & 0x30 == 0x30
    }
}
