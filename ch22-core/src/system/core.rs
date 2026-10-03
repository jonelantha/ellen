use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::{
    Clock,
    address_map::{AddressMap, FnAddressMap},
    runner::Runner,
};
use crate::address_spaces::{IOSpace, Ram, Rom};
use crate::devices::{RomSelect, SysViaBus};
use crate::sound_register_writes::SoundRegisterWrites;
use crate::video::Video;
use crate::{cpu::Cpu, devices::DeviceSpeed};

pub struct Core {
    clock: Clock,
    cpu: Cpu,
    ram: Ram,
    pub roms: [Rom; ROMS_LEN],
    pub io_space: IOSpace,
    pub ic32_latch: Rc<Cell<u8>>,
    rom_select_latch: Rc<Cell<usize>>,
    pub video: Video,
    sys_via_bus: Rc<RefCell<SysViaBus>>,
}

impl Default for Core {
    fn default() -> Self {
        let ic32_latch = Rc::<Cell<u8>>::default();

        let sys_via_bus = Rc::new(RefCell::new(SysViaBus::new(ic32_latch.clone())));

        Core {
            clock: Default::default(),
            cpu: Default::default(),
            ram: Default::default(),
            roms: Default::default(),
            io_space: Default::default(),
            ic32_latch,
            rom_select_latch: Default::default(),
            video: Default::default(),
            sys_via_bus,
        }
    }
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
        self.get_runner().reset();
    }

    pub fn run_one_field(&mut self) -> u64 {
        self.sys_via_bus
            .borrow_mut()
            .sound_mut()
            .start_field(self.clock.get_cycles());

        loop {
            let next_scanline_trigger = self.video.get_next_scanline_trigger();

            self.get_runner().run(next_scanline_trigger);

            self.video.process_scanline(
                self.ic32_latch.get(),
                |range| self.ram.slice(range),
                |vsync| self.io_space.on_vsync_change(vsync),
            );

            if self.video.is_field_complete() {
                return self.clock.get_cycles();
            }
        }
    }

    fn get_runner(&mut self) -> Runner<'_, impl AddressMap> {
        Runner::new(
            &mut self.clock,
            &mut self.ram,
            &self.roms,
            &mut self.io_space,
            &self.rom_select_latch,
            Self::address_map(),
            &mut self.cpu,
        )
    }

    pub fn get_sound_register_writes_start(&self) -> *const SoundRegisterWrites {
        self.sys_via_bus.borrow().sound().register_writes_ptr()
    }

    pub fn get_sys_via_bus(&self) -> Rc<RefCell<SysViaBus>> {
        self.sys_via_bus.clone()
    }
}

pub const OS_ROM: usize = 16;
pub const ROMS_LEN: usize = 17;
