use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::{
    Clock,
    address_map::{AddressMap, FnAddressMap},
    cpu_bus::CpuBus,
    runner::{Runner, RunnerTrait},
};
use crate::address_spaces::{IOSpace, Ram, Rom};
use crate::devices::{RomSelect, TimerDeviceList};
use crate::video::{
    Crtc, Field, VideoCRTCRegistersDevice, VideoRegisters, VideoULARegistersDevice,
};
use crate::{cpu::Cpu, devices::DeviceSpeed};

#[derive(Default)]
pub struct Core {
    cycles: u64,
    cpu: Cpu,
    ram: Ram,
    pub roms: [Rom; ROMS_LEN],
    pub io_space: IOSpace,
    pub video_field: Field,
    pub crtc: Crtc,
    pub ic32_latch: Rc<Cell<u8>>,
    pub field_counter: u8,
    pub rom_select_latch: Rc<Cell<usize>>,
    pub video_registers: Rc<RefCell<VideoRegisters>>,
    pub timer_devices: TimerDeviceList,
}

impl Core {
    pub fn setup(&mut self) {
        self.video_registers.borrow_mut().reset();

        self.crtc.init(&self.video_registers.borrow());

        self.io_space.add_device(
            &[
                0xfe00, 0xfe01, 0xfe02, 0xfe03, 0xfe04, 0xfe05, 0xfe06, 0xfe07,
            ],
            Box::new(VideoCRTCRegistersDevice::new(self.video_registers.clone())),
            None,
            DeviceSpeed::OneMhz,
        );

        self.io_space.add_device(
            &[0xfe20, 0xfe21, 0xfe22, 0xfe23],
            Box::new(VideoULARegistersDevice::new(self.video_registers.clone())),
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

        self.field_counter = 0;
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

    pub fn inc_field_counter(&mut self) {
        self.field_counter = self.field_counter.wrapping_add(1);
    }

    pub fn on_vsync_change(&mut self, vsync: bool) {
        self.io_space.on_vsync_change(vsync);
    }

    pub fn process_scanline(&mut self) {
        let registers = &self.video_registers.borrow();

        let snapshot_params = self.crtc.get_snapshot_params(registers);

        if snapshot_params.is_displayed {
            self.video_field.snapshot_scanline(
                snapshot_params.beam_scanline as usize,
                snapshot_params.address,
                snapshot_params.raster_address_even,
                snapshot_params.raster_address_odd,
                self.ic32_latch.get(),
                self.field_counter,
                registers,
                |range| self.ram.slice(range),
            );
        }

        self.crtc.advance_scanline(registers);
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

    fn with_runner(&mut self, run_fn: impl FnOnce(&mut dyn RunnerTrait)) -> u64 {
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

        self.cycles
    }
}

pub const OS_ROM: usize = 16;
pub const ROMS_LEN: usize = 17;
