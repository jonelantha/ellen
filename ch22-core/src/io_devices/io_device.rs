use crate::word::Word;

pub trait IODevice {
    fn read(&mut self, address: Word, cycles: u64) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: u64) -> bool;
    fn phase_2(&mut self, _cycles: u64) {}
    fn get_interrupt(&mut self, _cycles: u64) -> bool {
        false
    }
    fn set_interrupt(&mut self, _interrupt: bool) {}
}
