use std::cell::Cell;
use std::rc::Rc;

use super::{
    Clock,
    address_map::{AddressMap, FnAddressMap},
    cpu_bus::CpuBus,
    runner::{Runner, RunnerTrait},
};
use crate::address_spaces::{IOSpace, Ram, Rom};
use crate::devices::{RomSelect, TimerDeviceList};
use crate::video::{CRTCRangeType, Field, FieldLineAdditionalData};
use crate::{cpu::Cpu, devices::DeviceSpeed};

#[derive(Default)]
pub struct Core {
    cycles: u64,
    cpu: Cpu,
    ram: Ram,
    pub roms: [Rom; ROMS_LEN],
    pub io_space: IOSpace,
    pub video_field: Field,
    pub ic32_latch: Rc<Cell<u8>>,
    pub rom_select_latch: Rc<Cell<usize>>,
    pub timer_devices: TimerDeviceList,
}

impl Core {
    pub fn setup(&mut self) {
        self.rom_select_latch.set(15);

        self.io_space.add_device(
            &[0xfe30, 0xfe31, 0xfe32, 0xfe33],
            Box::new(RomSelect::new(self.rom_select_latch.clone())),
            None,
            DeviceSpeed::TwoMhz,
        );
    }

    fn address_map() -> impl AddressMap {
        FnAddressMap {
            read: |address, clock, ram, roms, io_space, rom_select_latch| match address.1 {
                ..0x80 => ram.read(address),
                0x80..0xc0 => roms[rom_select_latch.get()].read(address.rebased_to(0x80)),
                0xc0..0xfc => roms[OS_ROM].read(address.rebased_to(0xc0)),
                0xfc..0xff => io_space.read(address, clock),
                0xff.. => roms[OS_ROM].read(address.rebased_to(0xc0)),
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

    pub fn snapshot_char_data(
        &mut self,
        row_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        field_line_additional_data: FieldLineAdditionalData,
        required_type: CRTCRangeType,
    ) {
        self.video_field.snapshot_char_data(
            row_index,
            crtc_address,
            crtc_length,
            self.ic32_latch.get(),
            field_line_additional_data,
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

    fn with_runner<F>(&mut self, f: F) -> u64
    where
        F: FnOnce(&mut dyn RunnerTrait),
    {
        let clock = Clock::new(&mut self.cycles, &mut self.timer_devices);

        let cpu_bus = CpuBus::new(
            clock,
            &mut self.ram,
            &self.roms,
            &mut self.io_space,
            &self.rom_select_latch,
            Self::address_map(),
        );

        let mut runner = Runner {
            cpu_bus,
            cpu: &mut self.cpu,
        };

        f(&mut runner);

        self.cycles
    }
}

pub const OS_ROM: usize = 16;
pub const ROMS_LEN: usize = 17;
