use std::{cell::Cell, rc::Rc};

use crate::word::Word;

use super::device::Device;

#[cfg(test)]
mod tests;

pub struct SysViaStub<OnSoundRegisterWrite> {
    read: Box<dyn Fn(u16, u64) -> u64>,
    write: Box<dyn Fn(u16, u8, u8, u64) -> u64>,
    on_sound_register_write: OnSoundRegisterWrite,
    on_vsync_change: Box<dyn Fn(bool) -> u64>,
    handle_trigger: Box<dyn Fn(u64) -> u64>,
    trigger: Option<u64>,
    interrupt: bool,
    ic32_latch: Rc<Cell<u8>>,
    ora: u8,
    ddrb: u8,
}

impl<OnSoundRegisterWrite> SysViaStub<OnSoundRegisterWrite>
where
    OnSoundRegisterWrite: Fn(u64, u8),
{
    pub fn new(
        read: Box<dyn Fn(u16, u64) -> u64>,
        write: Box<dyn Fn(u16, u8, u8, u64) -> u64>,
        on_vsync_change: Box<dyn Fn(bool) -> u64>,
        handle_trigger: Box<dyn Fn(u64) -> u64>,
        ic32_latch: Rc<Cell<u8>>,
        on_sound_register_write: OnSoundRegisterWrite,
    ) -> Self {
        SysViaStub {
            read,
            write,
            on_sound_register_write,
            on_vsync_change,
            handle_trigger,
            trigger: None,
            interrupt: false,
            ic32_latch,
            ora: 0,
            ddrb: 0,
        }
    }
}

impl<OnSoundRegisterWrite> Device for SysViaStub<OnSoundRegisterWrite>
where
    OnSoundRegisterWrite: Fn(u64, u8),
{
    fn read(&mut self, address: Word, cycles: u64) -> u8 {
        self.set_params((self.read)(address.into(), cycles))
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: u64) -> bool {
        true
    }

    fn phase_2(&mut self, address: Word, value: u8, cycles: u64) {
        let sound_reg = match address.0 & 0x0f {
            0 => {
                if (self.ddrb & 0x0f) != 0x0f {
                    panic!(
                        "SysViaStub: Attempt to write to IC32 latch when DDRB is not set to output for all bits. DDRB: {:02x}",
                        self.ddrb
                    );
                }

                self.ic32_write(value)
            }
            1 | 15 => {
                self.ora = value;
                if self.ic32_latch.get() & 0x01 == 0 {
                    Some(value)
                } else {
                    None
                }
            }
            2 => {
                self.ddrb = value;
                None
            }
            _ => None,
        };

        if let Some(sound_reg) = sound_reg {
            (self.on_sound_register_write)(cycles, sound_reg);
        }

        self.set_params((self.write)(
            address.into(),
            value,
            self.ic32_latch.get(),
            cycles,
        ));
    }

    fn get_interrupt(&mut self, cycles: u64) -> bool {
        self.sync(cycles);

        self.interrupt
    }

    fn set_interrupt(&mut self, interrupt: bool) {
        self.interrupt = interrupt;
    }

    fn on_vsync_change(&mut self, vsync: bool) {
        self.set_params((self.on_vsync_change)(vsync));
    }
}

impl<OnSoundRegisterWrite> SysViaStub<OnSoundRegisterWrite>
where
    OnSoundRegisterWrite: Fn(u64, u8),
{
    fn sync(&mut self, cycles: u64) {
        if let Some(trigger) = self.trigger
            && trigger <= cycles
        {
            self.set_params((self.handle_trigger)(cycles));
        }
    }

    // Encoding format: [trig trig trig trig trig trig flags (value or ic32)]
    // The last byte contains either a value or ic32 data, depending on the JS_DEVICE_FLAG_VALUE_IS_IC32 flag.
    fn set_params(&mut self, params_and_value: u64) -> u8 {
        let [_, _, _, _, _, _, flags, value] = params_and_value.to_be_bytes();

        self.interrupt = flags & SYS_VIA_STUB_FLAG_INTERRUPT != 0;

        self.trigger = if flags & SYS_VIA_STUB_FLAG_HAS_TRIGGER != 0 {
            Some(params_and_value >> 16)
        } else {
            None
        };

        value
    }

    fn ic32_write(&mut self, value: u8) -> Option<u8> {
        let bit = value & 0x07;
        let old_value = self.ic32_latch.get();
        if value & 0x08 != 0 {
            self.ic32_latch.set(old_value | (1 << bit));
        } else {
            self.ic32_latch.set(old_value & !(1 << bit));
        }

        if old_value & 0x01 != 0 && self.ic32_latch.get() & 0x01 == 0 {
            Some(self.ora)
        } else {
            None
        }
    }
}

const SYS_VIA_STUB_FLAG_HAS_TRIGGER: u8 = 0x01;
const SYS_VIA_STUB_FLAG_INTERRUPT: u8 = 0x02;
