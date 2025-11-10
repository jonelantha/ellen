use std::cell::Cell;
use std::rc::Rc;

use crate::address_map::{AddressMap, DeviceSpeed, IODeviceID};
use crate::cpu::Cpu;
use crate::devices::{IODevice, TimerDevice, TimerDeviceID, TimerDeviceList};
use crate::interrupt_type::InterruptType;
use crate::system::system_runner::SystemRunner;
use crate::video::{CRTCRangeType, Field, VideoMemoryAccess};

#[derive(Default)]
pub struct SystemState {
    cpu: Cpu,
    video_field: Field,
    ic32_latch: Rc<Cell<u8>>,
    cycles: u64,
    timer_devices: TimerDeviceList,
    address_map: AddressMap,
}

impl SystemState {
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

        let ram = &self.address_map.ram;

        let first_ram_slice = ram.slice(first_ram_range);
        let second_ram_slice = second_ram_range.map(|range| ram.slice(range));

        self.video_field
            .set_char_data_line(row_index, first_ram_slice, second_ram_slice);
    }

    pub fn load_os_rom(&mut self, data: &[u8]) {
        self.address_map.os_rom.load(data);
    }

    pub fn load_paged_rom(&mut self, bank: u8, data: &[u8]) {
        self.address_map.paged_rom.load(bank, data);
    }

    pub fn add_io_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        speed: DeviceSpeed,
    ) -> IODeviceID {
        self.address_map
            .io_space
            .add_device(addresses, device, interrupt_type, speed)
    }

    pub fn add_timer_device(&mut self, timer_device: Box<dyn TimerDevice>) -> TimerDeviceID {
        self.timer_devices.add_device(timer_device)
    }

    pub fn reset(&mut self) {
        let mut system_runner = SystemRunner::new(
            &mut self.cycles,
            &mut self.cpu,
            &mut self.timer_devices,
            &mut self.address_map,
        );

        system_runner.reset();
    }

    pub fn run(&mut self, until: u64) -> u64 {
        let mut system_runner = SystemRunner::new(
            &mut self.cycles,
            &mut self.cpu,
            &mut self.timer_devices,
            &mut self.address_map,
        );

        system_runner.run(until);

        self.cycles
    }

    pub fn set_device_interrupt(&mut self, device_id: IODeviceID, interrupt: bool) {
        self.address_map
            .io_space
            .set_interrupt(device_id, interrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.timer_devices.set_device_trigger(device_id, trigger);
    }

    pub fn clone_ic32_latch(&self) -> Rc<Cell<u8>> {
        Rc::clone(&self.ic32_latch)
    }
}
