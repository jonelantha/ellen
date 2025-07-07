use std::cell::Cell;
use std::rc::Rc;

use crate::clock::Clock;
use crate::devices_lib::addressable_device::AddressableDevice;
use crate::word::Word;

pub struct RomSelect {
    active_rom: Rc<Cell<usize>>,
}

impl RomSelect {
    pub fn new(active_rom: Rc<Cell<usize>>) -> Self {
        RomSelect { active_rom }
    }
}

impl AddressableDevice for RomSelect {
    fn read(&mut self, _address: Word, _clock: &mut Clock) -> u8 {
        self.active_rom.get() as u8
    }

    fn write(&mut self, _address: Word, value: u8, _clock: &mut Clock) -> bool {
        self.active_rom.set(value as usize);

        false
    }
}
