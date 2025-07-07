pub trait TimerDevice {
    fn sync(&mut self, _cycles: u64);
    fn set_trigger(&mut self, trigger: Option<u64>);
}
