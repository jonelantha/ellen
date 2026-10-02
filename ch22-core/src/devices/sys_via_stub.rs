use std::{cell::Cell, rc::Rc};

use crate::word::Word;

use super::device::Device;

#[cfg(test)]
mod tests;

pub struct SysViaStub<Sound> {
    bus: SysViaBus<Sound>,
    read: Box<dyn Fn(u16, u64) -> u64>,
    write: Box<dyn Fn(u16, u8, u8, u64) -> u64>,
    on_vsync_change: Box<dyn Fn(bool) -> u64>,
    handle_trigger: Box<dyn Fn(u64) -> u64>,
    trigger: Option<u64>,
    interrupt: bool,
    data_registers: ViaDataRegisters,
}

impl<OnSoundRegisterWrite> SysViaStub<SN76496Stub<OnSoundRegisterWrite>>
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
        let bus = SysViaBus::new(SN76496Stub::new(on_sound_register_write), ic32_latch);

        SysViaStub {
            bus,
            read,
            write,
            on_vsync_change,
            handle_trigger,
            trigger: None,
            interrupt: false,
            data_registers: ViaDataRegisters::default(),
        }
    }
}

impl<Sound: SoundChip> Device for SysViaStub<Sound> {
    fn read(&mut self, address: Word, cycles: u64) -> u8 {
        self.set_params((self.read)(address.into(), cycles))
    }

    fn write(&mut self, _address: Word, _value: u8, _cycles: u64) -> bool {
        true
    }

    fn phase_2(&mut self, address: Word, value: u8, cycles: u64) {
        match address.0 & 0x0f {
            0 => {
                self.data_registers.orb = value;
                self.update_bus(cycles);
            }
            1 | 15 => {
                self.data_registers.ora = value;
                self.update_bus(cycles);
            }
            2 => {
                self.data_registers.ddrb = value;
                self.update_bus(cycles);
            }
            3 => {
                self.data_registers.ddra = value;
                self.update_bus(cycles);
            }
            _ => (),
        }

        self.set_params((self.write)(
            address.into(),
            value,
            self.bus.ic32_latch.get(),
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

impl<Sound: SoundChip> SysViaStub<Sound> {
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

    fn update_bus(&mut self, cycles: u64) {
        self.bus.update(
            ViaPortState {
                value: self.data_registers.ora,
                output_mask: self.data_registers.ddra,
            },
            ViaPortState {
                value: self.data_registers.orb,
                output_mask: self.data_registers.ddrb,
            },
            cycles,
        );
    }
}

const SYS_VIA_STUB_FLAG_HAS_TRIGGER: u8 = 0x01;
const SYS_VIA_STUB_FLAG_INTERRUPT: u8 = 0x02;

#[derive(Default)]
struct ViaDataRegisters {
    ora: u8,
    ddra: u8,
    orb: u8,
    ddrb: u8,
}

struct ViaPortState {
    value: u8,
    output_mask: u8,
}

impl ViaPortState {
    fn resolve_floating(&self) -> u8 {
        // floating lines treated as high
        (self.value & self.output_mask) | !self.output_mask
    }
}

struct SysViaBus<Sound> {
    sound: Sound,
    ic32_latch: IC32Latch,
}

impl<Sound: SoundChip> SysViaBus<Sound> {
    pub fn new(sound: Sound, ic32_latch: Rc<Cell<u8>>) -> Self {
        SysViaBus {
            sound,
            ic32_latch: IC32Latch::new(ic32_latch),
        }
    }

    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64) {
        let port_a_data = port_a.resolve_floating();
        let port_b_data = port_b.resolve_floating();

        let latch_output = self.ic32_latch.update(port_b_data & 0x0f);

        let latch_output_line_0 = latch_output & 0x01 == 0x01;

        let sound_written = self.sound.update(latch_output_line_0, port_a_data, cycles);

        if sound_written {
            self.debug_check_sound_mask(port_a.output_mask, cycles);
        }
    }

    fn debug_check_sound_mask(&self, output_mask: u8, cycles: u64) {
        if output_mask != 0xff {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!(
                    "sound write output_mask == {:#04x} {:?}",
                    output_mask, cycles
                )
                .into(),
            );
        }
    }
}

pub struct IC32Latch {
    latch: Rc<Cell<u8>>,
}

impl IC32Latch {
    fn new(latch: Rc<Cell<u8>>) -> Self {
        IC32Latch { latch }
    }

    fn get(&self) -> u8 {
        self.latch.get()
    }

    fn update(&mut self, data: u8) -> u8 {
        let bit = 1 << (data & 0x07);

        if data & 0x08 != 0 {
            self.latch.set(self.latch.get() | bit);
        } else {
            self.latch.set(self.latch.get() & !bit);
        };

        self.latch.get()
    }
}

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
    fn new(on_sound_register_write: OnSoundRegisterWrite) -> Self {
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
