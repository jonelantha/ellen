use super::device::Device;
use crate::word::Word;

pub struct StaticDevice {
    pub read_value: u8,
    pub panic_on_write: bool,
}

impl Device for StaticDevice {
    fn read(&mut self, _address: Word, _cycles: u64) -> u8 {
        self.read_value
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: u64) -> bool {
        if self.panic_on_write {
            panic!();
        }
        false
    }
}
