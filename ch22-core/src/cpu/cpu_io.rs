use crate::interrupt_type::InterruptType;
use crate::word::Word;

pub mod cpu_io_mock;

pub trait CpuIO {
    fn phantom_read(&mut self, address: Word);
    fn read(&mut self, address: Word) -> u8;
    fn write(&mut self, address: Word, value: u8);
    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool;
}
