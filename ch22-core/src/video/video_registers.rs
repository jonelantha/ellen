#[derive(Default, Copy, Clone)]
#[repr(C, packed)]
pub struct VideoRegisters {
    pub ula_control: u8,
    pub ula_palette: u64,
}

impl VideoRegisters {
    pub fn set_ula_palette(&mut self, entry: u8, value: u8) {
        let shift = entry * 4;

        self.ula_palette &= !(0x0f << shift);
        self.ula_palette |= ((value & 0x0f) as u64) << shift;
    }

    pub fn is_teletext(&self) -> bool {
        (self.ula_control & 0x02) != 0
    }
}
