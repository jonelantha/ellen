use crate::video::VideoRegisters;

#[derive(Default)]
pub struct VSyncControl {
    line_countdown: u8,
}

impl VSyncControl {
    pub fn reset(&mut self) {
        self.line_countdown = 0;
    }

    pub fn start_vsync_period(&mut self, registers: &VideoRegisters) -> bool {
        if self.is_in_vsync() {
            false
        } else {
            self.line_countdown = registers.r3_v_sync_width();

            true
        }
    }

    pub fn advance_scanline(&mut self) {
        if self.line_countdown > 0 {
            self.line_countdown -= 1;
        }
    }

    pub fn is_in_vsync(&self) -> bool {
        self.line_countdown > 0
    }
}
