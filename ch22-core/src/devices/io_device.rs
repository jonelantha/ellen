use crate::word::Word;

pub trait Ch22IODevice {
    fn read(&mut self, address: Word, cycles: u32, interrupt: &mut u8) -> u8;
    fn write(&mut self, address: Word, value: u8, cycles: u32, interrupt: &mut u8) -> bool;
    fn phase_2(&mut self, address: Word, cycles: u32, interrupt: &mut u8);
    fn is_slow(&self) -> bool;
    fn sync(&mut self, cycles: u32, interrupt: &mut u8);
    fn set_trigger(&mut self, trigger: Option<u32>);
}
