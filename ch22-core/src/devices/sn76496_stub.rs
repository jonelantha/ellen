use std::{cell::RefCell, rc::Rc};

use crate::sound_register_writes::SoundRegisterWrites;

pub trait SoundChip {
    /// Called with the current level of the chip's inputs whenever anything
    /// upstream may have changed them, so most calls change nothing.
    ///
    /// The chip only sees input levels, and right now only logs writes when
    /// enabled and something changes - in the future we move to a full event
    /// logging model (logging disabled events) and let the renderer decide
    /// when a write occured or reoccured
    ///
    /// Returns true if this update latched a register write.
    fn update(&mut self, write_enable_active_low: bool, data: u8, cycles: u64) -> bool;
}

pub struct SN76496Stub {
    sound_register_writes: Rc<RefCell<SoundRegisterWrites>>,
    previous_data: Option<u8>,
}

impl SN76496Stub {
    pub fn new(sound_register_writes: Rc<RefCell<SoundRegisterWrites>>) -> Self {
        SN76496Stub {
            sound_register_writes,
            previous_data: None,
        }
    }
}

impl SoundChip for SN76496Stub {
    fn update(&mut self, write_enable_active_low: bool, data: u8, cycles: u64) -> bool {
        if !write_enable_active_low {
            if self
                .previous_data
                .is_none_or(|previous_data| previous_data != data)
            {
                self.sound_register_writes.borrow_mut().push(cycles, data);

                self.previous_data = Some(data);

                return true;
            }
        } else {
            self.previous_data = None;
        }

        false
    }
}
