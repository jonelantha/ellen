use std::cell::Cell;
use std::rc::Rc;

use super::{Clock, cpu_bus::CpuBus};
use crate::address_spaces::{IOSpace, PagedRom, Ram, Rom};
use crate::cpu::Cpu;
use crate::devices::TimerDeviceList;
use crate::video::{CRTCRangeType, Field};
use crate::word::Word;

#[derive(Default)]
pub struct SystemComponents {
    cycles: u64,
    cpu: Cpu,
    ram: Ram,
    paged_rom: PagedRom,
    io_space: IOSpace,
    os_rom: Rom,
    video_field: Field,
    ic32_latch: Rc<Cell<u8>>,
    timer_devices: TimerDeviceList,
}

impl SystemComponents {
    pub fn paged_rom(&mut self) -> &mut PagedRom {
        &mut self.paged_rom
    }

    pub fn io_space(&mut self) -> &mut IOSpace {
        &mut self.io_space
    }

    pub fn os_rom(&mut self) -> &mut Rom {
        &mut self.os_rom
    }

    pub fn video_field(&mut self) -> &mut Field {
        &mut self.video_field
    }

    pub fn timer_devices(&mut self) -> &mut TimerDeviceList {
        &mut self.timer_devices
    }

    pub fn snapshot_char_data(
        &mut self,
        row_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        required_type: CRTCRangeType,
    ) {
        self.video_field.snapshot_char_data(
            row_index,
            crtc_address,
            crtc_length,
            self.ic32_latch.get(),
            required_type,
            |range| self.ram.slice(range),
        );
    }

    pub fn reset(&mut self) {
        self.with_runner(|cpu, cpu_bus| {
            cpu.reset(cpu_bus);
        });
    }

    pub fn run(&mut self, until: u64) -> u64 {
        self.with_runner(|cpu, cpu_bus| {
            while cpu_bus.get_clock().get_cycles() < until {
                cpu.handle_next_instruction(cpu_bus);
            }
        })
    }

    pub fn with_runner<F>(&mut self, f: F) -> u64
    where
        F: FnOnce(&mut Cpu, &mut CpuBus),
    {
        let clock = Clock::new(&mut self.cycles, &mut self.timer_devices);

        let address_map = AddressMap {
            ram: &mut self.ram,
            paged_rom: &mut self.paged_rom,
            io_space: &mut self.io_space,
            os_rom: &mut self.os_rom,
        };

        let mut cpu_bus = CpuBus::new(clock, address_map);

        f(&mut self.cpu, &mut cpu_bus);

        self.cycles
    }

    pub fn clone_ic32_latch(&self) -> Rc<Cell<u8>> {
        Rc::clone(&self.ic32_latch)
    }
}

pub struct AddressMap<'a> {
    ram: &'a mut Ram,
    paged_rom: &'a mut PagedRom,
    io_space: &'a mut IOSpace,
    os_rom: &'a mut Rom,
}

impl AddressMap<'_> {
    pub fn io_space_mut(&mut self) -> &mut IOSpace {
        self.io_space
    }

    pub fn read(&mut self, address: Word, clock: &mut Clock) -> u8 {
        match address.1 {
            ..0x80 => self.ram.read(address),
            0x80..0xc0 => self.paged_rom.read(address.rebased_to(0x80)),
            0xc0..0xfc => self.os_rom.read(address.rebased_to(0xc0)),
            0xfc..0xff => self.io_space.read(address, clock),
            0xff.. => self.os_rom.read(address.rebased_to(0xc0)),
        }
    }

    pub fn write(&mut self, address: Word, value: u8, clock: &mut Clock) {
        match address.1 {
            ..0x80 => self.ram.write(address, value),
            0x80..0xc0 => (), // paged rom
            0xc0..0xfc => (), // os rom
            0xfc..0xff => self.io_space.write(address, value, clock),
            0xff.. => (), // os rom
        }
    }
}
