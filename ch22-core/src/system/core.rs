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
use crate::video::Video;
use crate::{cpu::Cpu, devices::DeviceSpeed};

#[derive(Default)]
pub struct Core {
    cycles: u64,
    cpu: Cpu,
    ram: Ram,
    pub roms: [Rom; ROMS_LEN],
    pub io_space: IOSpace,
    pub ic32_latch: Rc<Cell<u8>>,
    rom_select_latch: Rc<Cell<usize>>,
    pub timer_devices: TimerDeviceList,
    pub video: Video,
}

impl Core {
    pub fn setup(&mut self) {
        self.video.init();

        self.io_space.add_device(
            &[
                0xfe00, 0xfe01, 0xfe02, 0xfe03, 0xfe04, 0xfe05, 0xfe06, 0xfe07,
            ],
            Box::new(self.video.create_crtc_registers_device()),
            None,
            DeviceSpeed::OneMhz,
        );

        self.io_space.add_device(
            &[0xfe20, 0xfe21, 0xfe22, 0xfe23],
            Box::new(self.video.create_ula_registers_device()),
            None,
            DeviceSpeed::OneMhz,
        );

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

    pub fn reset(&mut self) {
        self.with_runner(|runner| {
            runner.reset();
        });
    }

    pub fn run_one_field(&mut self) -> u64 {
        loop {
            self.run(self.video.get_next_scanline_trigger());

            let is_field_complete = self.process_scanline();

            if is_field_complete {
                return self.cycles;
            }
        }
    }

    fn run(&mut self, until: u64) {
        self.with_runner(|runner| {
            runner.run(until);
        })
    }

    fn with_runner(&mut self, run_fn: impl FnOnce(&mut dyn RunnerTrait)) {
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

        run_fn(&mut runner);
    }

    fn process_scanline(&mut self) -> bool {
        self.video.process_scanline(
            self.ic32_latch.get(),
            |range| self.ram.slice(range),
            |vsync| self.io_space.on_vsync_change(vsync),
        )
    }
}

pub const OS_ROM: usize = 16;
pub const ROMS_LEN: usize = 17;
