use crate::word::Word;

pub trait Ch22IODevice {
    fn read(&mut self, address: Word, cycles: u32, interrupt: &mut u8) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: u32, interrupt: &mut u8) -> bool;
    fn phase_2(&mut self, _cycles: u32, _interrupt: &mut u8) {}
    fn is_slow(&self) -> bool;
    fn sync(&mut self, _cycles: u32, _interrupt: &mut u8) {}
    fn set_trigger(&mut self, _trigger: Option<u32>) {}
    fn wrap_trigger(&mut self, _wrap: u32) {}
}
