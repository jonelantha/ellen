use crate::video::VideoRegisters;

#[derive(Default)]
pub struct AddressControl {
    address: u16,
}

impl AddressControl {
    pub fn reset(&mut self, registers: &VideoRegisters) {
        self.address = registers.r12_r13_screen_address();
    }

    pub fn advance_char_row(&mut self, registers: &VideoRegisters) {
        self.address = (self.address + registers.crtc_r1_horizontal_displayed as u16) & 0x3FFF;
    }

    pub fn get_address(&self) -> u16 {
        self.address
    }
}
