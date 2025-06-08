pub trait Ch22Device {
    fn read(&mut self, address: u16, cycles: u32) -> u8;
    fn write(&mut self, address: u16, value: u8, cycles: u32) -> bool;
    fn phase_2(&mut self, cycles: u32);
    fn is_slow(&self, address: u16) -> bool;
}
