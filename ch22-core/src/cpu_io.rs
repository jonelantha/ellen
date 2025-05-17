use crate::word::Word;

pub trait CpuIO {
    fn phantom_read(&mut self, address: Word);
    fn read(&mut self, address: Word) -> u8;
    fn write(&mut self, address: Word, value: u8);
    fn complete(&mut self);
    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool);
}
