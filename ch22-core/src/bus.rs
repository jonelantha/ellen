pub trait BusTrait {
    fn phantom_read(&mut self, address: u16);
    fn read(&mut self, address: u16, op: CycleOp) -> u8;
    fn write(&mut self, address: u16, value: u8, op: CycleOp);
    fn complete(&self);
}

#[derive(PartialEq, Clone, Copy)]
pub enum CycleOp {
    Sync,
    CheckInterrupt,
    None,
}
