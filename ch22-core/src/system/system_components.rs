use std::cell::Cell;
use std::rc::Rc;

use super::{Clock, cpu_bus::CpuBus};
use crate::address_spaces::{IOSpace, PagedRom, Ram, Rom};
use crate::cpu::{Cpu, InterruptType};
use crate::devices::{
    DeviceSpeed, IODevice, IODeviceID, RomSelect, TimerDevice, TimerDeviceID, TimerDeviceList,
};
use crate::video::{CRTCRangeType, Field, VideoMemoryAccess};
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
    pub fn setup(&mut self) {
        self.io_space.add_device(
            &[0xfe30, 0xfe31, 0xfe32, 0xfe33],
            Box::new(RomSelect::new(self.paged_rom.get_active_rom())),
            None,
            DeviceSpeed::TwoMhz,
        );
    }

    pub fn video_field_start(&self) -> *const Field {
        &self.video_field as *const Field
    }

    pub fn snapshot_char_data(
        &mut self,
        row_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        required_type: CRTCRangeType,
    ) {
        if crtc_length == 0 {
            self.video_field.set_blank_line(row_index);
            return;
        }

        let video_type = VideoMemoryAccess::get_crtc_range_type(crtc_address, crtc_length);

        if video_type != required_type {
            self.video_field.set_blank_line(row_index);
            return;
        }

        let (first_ram_range, second_ram_range) = VideoMemoryAccess::translate_crtc_range(
            crtc_address,
            crtc_length,
            self.ic32_latch.get(),
        );

        let ram = &self.ram;

        let first_ram_slice = ram.slice(first_ram_range);
        let second_ram_slice = second_ram_range.map(|range| ram.slice(range));

        self.video_field
            .set_char_data_line(row_index, first_ram_slice, second_ram_slice);
    }

    pub fn load_os_rom(&mut self, data: &[u8]) {
        self.os_rom.load(data);
    }

    pub fn load_paged_rom(&mut self, bank: u8, data: &[u8]) {
        self.paged_rom.load(bank, data);
    }

    pub fn add_io_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        speed: DeviceSpeed,
    ) -> IODeviceID {
        self.io_space
            .add_device(addresses, device, interrupt_type, speed)
    }

    pub fn add_timer_device(&mut self, timer_device: Box<dyn TimerDevice>) -> TimerDeviceID {
        self.timer_devices.add_device(timer_device)
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

    pub fn set_device_interrupt(&mut self, device_id: IODeviceID, interrupt: bool) {
        self.io_space.set_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.timer_devices.set_device_trigger(device_id, trigger);
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
