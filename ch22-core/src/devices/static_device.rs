use crate::clock::Clock;
use crate::devices_lib::addressable_device::AddressableDevice;
use crate::word::Word;

pub struct StaticDevice {
    pub read_value: u8,
    pub panic_on_write: bool,
}

impl AddressableDevice for StaticDevice {
    fn read(&mut self, _address: Word, _clock: &mut Clock) -> u8 {
        self.read_value
    }

    fn write(&mut self, _address: Word, _value: u8, _clock: &mut Clock) -> bool {
        if self.panic_on_write {
            panic!();
        }
        false
    }
}
