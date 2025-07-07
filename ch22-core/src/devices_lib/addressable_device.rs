use crate::clock::Clock;
use crate::word::Word;

pub trait AddressableDevice {
    fn read(&mut self, address: Word, _clock: &mut Clock) -> u8;
    fn write(&mut self, _address: Word, _value: u8, _clock: &mut Clock) -> bool {
        false
    }
    fn phase_2(&mut self, _address: Word, _cycles: u64) {}
    fn get_interrupt(&mut self, _cycles: u64) -> bool {
        false
    }
    fn set_interrupt(&mut self, _interrupt: bool) {}
}
