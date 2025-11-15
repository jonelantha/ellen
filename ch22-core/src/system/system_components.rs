use std::cell::Cell;
use std::rc::Rc;

use super::{
    Clock,
    address_map::{AddressMap, FnAddressMap},
    cpu_bus::CpuBus,
    system_runner::{SystemRunner, SystemRunnerTrait},
};
use crate::address_spaces::{IOSpace, PagedRom, Ram, Rom};
use crate::devices::{RomSelect, TimerDeviceList};
use crate::video::{CRTCRangeType, Field};
use crate::{cpu::Cpu, devices::DeviceSpeed};

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
    pub fn setup(&mut self) {
        self.io_space.add_device(
            &[0xfe30, 0xfe31, 0xfe32, 0xfe33],
            Box::new(RomSelect::new(self.paged_rom.get_active_rom())),
            None,
            DeviceSpeed::TwoMhz,
        );
    }

    fn address_map() -> impl AddressMap {
        FnAddressMap {
            read: |address, clock, ram, paged_rom, io_space, os_rom| match address.1 {
                ..0x80 => ram.read(address),
                0x80..0xc0 => paged_rom.read(address.rebased_to(0x80)),
                0xc0..0xfc => os_rom.read(address.rebased_to(0xc0)),
                0xfc..0xff => io_space.read(address, clock),
                0xff.. => os_rom.read(address.rebased_to(0xc0)),
            },
            write: |address, value, clock, ram, io_space| {
                match address.1 {
                    ..0x80 => ram.write(address, value),
                    0x80..0xc0 => (), // paged rom
                    0xc0..0xfc => (), // os rom
                    0xfc..0xff => io_space.write(address, value, clock),
                    0xff.. => (), // os rom
                }
            },
        }
    }

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
        self.with_runner(|runner| {
            runner.reset();
        });
    }

    pub fn run(&mut self, until: u64) -> u64 {
        self.with_runner(|runner| {
            runner.run(until);
        })
    }

    pub fn with_runner<F>(&mut self, f: F) -> u64
    where
        F: FnOnce(&mut dyn SystemRunnerTrait),
    {
        let clock = Clock::new(&mut self.cycles, &mut self.timer_devices);

        let cpu_bus = CpuBus::new(
            clock,
            &mut self.ram,
            &mut self.paged_rom,
            &mut self.io_space,
            &mut self.os_rom,
            Self::address_map(),
        );

        let mut runner = SystemRunner::new(cpu_bus, &mut self.cpu);

        f(&mut runner);

        self.cycles
    }

    pub fn clone_ic32_latch(&self) -> Rc<Cell<u8>> {
        Rc::clone(&self.ic32_latch)
    }
}
