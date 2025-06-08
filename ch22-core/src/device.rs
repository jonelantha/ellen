pub trait Ch22Device {
    fn read(&mut self, address: u16, machine_cycles: u32) -> u8;
    fn write(&mut self, address: u16, value: u8, machine_cycles: u32) -> bool;
    fn phase_2(&mut self, machine_cycles: u32);
    fn is_slow(&self, address: u16) -> bool;
}
