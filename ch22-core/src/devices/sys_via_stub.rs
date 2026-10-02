use std::{cell::Cell, rc::Rc};

use crate::word::Word;

use super::device::Device;
use super::via_port_connections::{ViaPortConnections, ViaPortState};

#[cfg(test)]
mod tests;

pub struct SysViaStub<PortConnections> {
    port_connections: PortConnections,
    ic32_latch: Rc<Cell<u8>>,
    read: Box<dyn Fn(u16, u64) -> u64>,
    write: Box<dyn Fn(u16, u8, u8, u64) -> u64>,
    on_vsync_change: Box<dyn Fn(bool) -> u64>,
    handle_trigger: Box<dyn Fn(u64) -> u64>,
    trigger: Option<u64>,
    interrupt: bool,
    data_registers: ViaDataRegisters,
}

impl<PortConnections: ViaPortConnections> SysViaStub<PortConnections> {
    pub fn new(
        read: Box<dyn Fn(u16, u64) -> u64>,
        write: Box<dyn Fn(u16, u8, u8, u64) -> u64>,
        on_vsync_change: Box<dyn Fn(bool) -> u64>,
        handle_trigger: Box<dyn Fn(u64) -> u64>,
        ic32_latch: Rc<Cell<u8>>,
        port_connections: PortConnections,
    ) -> Self {
        SysViaStub {
            port_connections,
            ic32_latch,
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

impl<PortConnections: ViaPortConnections> Device for SysViaStub<PortConnections> {
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

impl<PortConnections: ViaPortConnections> SysViaStub<PortConnections> {
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
        self.port_connections.update(
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
