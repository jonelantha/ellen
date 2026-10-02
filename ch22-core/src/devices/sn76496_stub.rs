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

pub struct SN76496Stub<OnSoundRegisterWrite> {
    on_sound_register_write: OnSoundRegisterWrite,
    previous_data: Option<u8>,
}

impl<OnSoundRegisterWrite> SN76496Stub<OnSoundRegisterWrite> {
    pub fn new(on_sound_register_write: OnSoundRegisterWrite) -> Self {
        SN76496Stub {
            on_sound_register_write,
            previous_data: None,
        }
    }
}

impl<OnSoundRegisterWrite> SoundChip for SN76496Stub<OnSoundRegisterWrite>
where
    OnSoundRegisterWrite: Fn(u64, u8),
{
    fn update(&mut self, write_enable_active_low: bool, data: u8, cycles: u64) -> bool {
        if !write_enable_active_low {
            if self
                .previous_data
                .is_none_or(|previous_data| previous_data != data)
            {
                (self.on_sound_register_write)(cycles, data);

                self.previous_data = Some(data);

                return true;
            }
        } else {
            self.previous_data = None;
        }

        false
    }
}
