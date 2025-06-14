use crate::word::Word;

pub trait Ch22IODevice {
    fn read(&mut self, address: Word, cycles: u32) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: u32) -> bool;
    fn phase_2(&mut self, address: Word, cycles: u32);
    fn is_slow(&self) -> bool;
    fn get_nmi(&mut self, cycles: u32) -> bool;
}
