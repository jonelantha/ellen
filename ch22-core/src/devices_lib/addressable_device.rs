use crate::clock::Clock;
use crate::word::Word;

pub trait AddressableDevice {
    fn read(&mut self, address: Word, cycles: &mut Clock) -> u8;
    fn write(&mut self, _address: Word, _value: u8, _cycles: &mut Clock) -> bool {
        false
    }
    fn phase_2(&mut self, _address: Word, _cycles: u64) {}
}
