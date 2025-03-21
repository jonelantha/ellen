use crate::word::Word;

pub trait Bus {
    fn phantom_read(&mut self, address: Word);
    fn read(&mut self, address: Word, op: CycleOp) -> u8;
    fn write(&mut self, address: Word, value: u8, op: CycleOp);
    fn complete(&self);
}

#[derive(PartialEq, Clone, Copy)]
pub enum CycleOp {
    Sync,
    CheckInterrupt,
    None,
}
