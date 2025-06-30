use crate::word::Word;

pub trait Ch22IODevice {
    fn read(&mut self, address: Word, cycles: u32) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: u32) -> bool;
    fn phase_2(&mut self, _cycles: u32) {}
    fn sync(&mut self, _cycles: u32) {}
    fn set_trigger(&mut self, _trigger: Option<u32>) {}
    fn wrap_trigger(&mut self, _wrap: u32) {}
    fn get_interrupt(&mut self, _cycles: u32) -> bool {
        false
    }
    fn set_interrupt(&mut self, _interrupt: bool) {}
}
