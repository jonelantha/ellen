use crate::word::Word;

pub trait Ch22Device {
    fn read(&mut self, address: Word, cycles: &mut u64) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: &mut u64) -> bool;
    fn phase_2(&mut self, _address: Word, _cycles: u64) {}
}
