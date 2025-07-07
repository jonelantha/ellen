pub trait TimerDevice {
    fn sync(&mut self, _cycles: u64) -> Option<u64>;
}
