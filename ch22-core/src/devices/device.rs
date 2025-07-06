use crate::word::Word;

pub trait Ch22Device {
    fn read(&mut self, address: Word, cycles: &mut u64) -> u8;
    fn write(&mut self, _address: Word, _value: u8, _cycles: &mut u64) -> bool {
        false
    }
    fn phase_2(&mut self, _address: Word, _cycles: u64) {}
}
