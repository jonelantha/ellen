use crate::word::Word;

pub trait Ch22Device {
    fn read(&mut self, address: Word, cycles: &mut u32) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: &mut u32) -> bool;
    fn phase_2(&mut self, address: Word, cycles: u32);
}
