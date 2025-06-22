use crate::word::Word;

use super::io_device::Ch22IODevice;

pub struct Ch22ConstantDevice {
    pub read_value: u8,
    pub is_slow: bool,
    pub panic_on_write: bool,
}

impl Ch22IODevice for Ch22ConstantDevice {
    fn read(&mut self, _address: Word, _cycles: u32, _interrupt: &mut u8) -> u8 {
        self.read_value
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: u32, _interrupt: &mut u8) -> bool {
        if self.panic_on_write {
            panic!();
        }
        false
    }

    fn phase_2(&mut self, _address: Word, _cycles: u32, _interrupt: &mut u8) {}

    fn is_slow(&self) -> bool {
        self.is_slow
    }

    fn sync(&mut self, _cycles: u32, _interrupt: &mut u8) {}

    fn set_trigger(&mut self, _trigger: Option<u32>) {}
}
