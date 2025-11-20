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

        self.crtc_registers[0] = 127;
        self.crtc_registers[1] = 80;
        self.crtc_registers[2] = 98;
        self.crtc_registers[3] = 0x28;
        self.crtc_registers[4] = 38;
        self.crtc_registers[5] = 0;
        self.crtc_registers[6] = 32;
        self.crtc_registers[7] = 34;
        self.crtc_registers[8] = 0;
        self.crtc_registers[9] = 7;
        self.crtc_registers[10] = 0;
        self.crtc_registers[11] = 0;
        self.crtc_registers[12] = 6;
        self.crtc_registers[13] = 0;
        self.crtc_registers[14] = 0;
        self.crtc_registers[15] = 0;
    }

    pub fn set_ula_palette(&mut self, entry: u8, value: u8) {
        let shift = entry * 4;

        self.ula_palette &= !(0x0f << shift);
        self.ula_palette |= ((value & 0x0f) as u64) << shift;
    }

    pub fn is_teletext(&self) -> bool {
        (self.ula_control & 0x02) != 0
    }
}
