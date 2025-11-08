use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::address_map::AddressMap;
use crate::address_map::io_space::DeviceSpeed;
use crate::address_map::io_space::io_device_list::IODeviceID;
use crate::clock::timer_device_list::TimerDeviceID;
use crate::clock::timer_device_list::TimerDeviceList;
use crate::cpu::*;
use crate::cycle_manager::*;
use crate::devices::js_io_device::JsIODevice;
use crate::devices::js_timer_device::*;
use crate::devices::static_device::StaticDevice;
use crate::interrupt_type::InterruptType;
use crate::utils;
use crate::video::field_data::Field;
use crate::video::video_memory_access::CRTCRangeType;
use crate::video::video_memory_access::VideoMemoryAccess;
use std::cell::Cell;
use std::mem::size_of;
use std::rc::Rc;

#[wasm_bindgen]
#[derive(Default)]
pub struct System {
    cpu: Cpu,
    video_field: Field,
    ic32_latch: Rc<Cell<u8>>,
    cycles: u64,
    timer_devices: TimerDeviceList,
    address_map: AddressMap,
}

#[wasm_bindgen]
impl System {
    pub fn new() -> System {
        utils::set_panic_hook();

        System::default()
    }

    pub fn video_field_start(&self) -> *const Field {
        &self.video_field as *const Field
    }

    pub fn video_field_size(&self) -> usize {
        size_of::<Field>()
    }

    pub fn snapshot_char_data(
        &mut self,
        row_index: usize,
        crtc_address: u16,
        crtc_length: u8,
        is_teletext: bool,
    ) {
        if crtc_length == 0 {
            self.video_field.set_blank_line(row_index);
            return;
        }

        let video_type = VideoMemoryAccess::get_crtc_range_type(crtc_address, crtc_length);

        let required_type = match is_teletext {
            true => CRTCRangeType::Teletext,
            false => CRTCRangeType::HiRes,
        };

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

    pub fn add_static_device(
        &mut self,
        addresses: &[u16],
        read_value: u8,
        one_mhz: bool,
        panic_on_write: bool,
    ) -> IODeviceID {
        let speed = match one_mhz {
            true => DeviceSpeed::OneMhz,
            false => DeviceSpeed::TwoMhz,
        };

        self.address_map.io_space.add_device(
            addresses,
            Box::new(StaticDevice {
                read_value,
                panic_on_write,
            }),
            None,
            speed,
        )
    }

    pub fn add_js_io_device(
        &mut self,
        addresses: &[u16],
        js_read: Function,
        js_write: Function,
        js_handle_trigger: Function,
        flags: u8,
    ) -> IODeviceID {
        let interrupt_type = match flags & (JS_DEVICE_IRQ | JS_DEVICE_NMI) {
            JS_DEVICE_IRQ => Some(InterruptType::IRQ),
            JS_DEVICE_NMI => Some(InterruptType::NMI),
            _ => None,
        };

        let speed = match flags & JS_DEVICE_ONE_MHZ {
            JS_DEVICE_ONE_MHZ => DeviceSpeed::OneMhz,
            _ => DeviceSpeed::TwoMhz,
        };

        self.address_map.io_space.add_device(
            addresses,
            Box::new(JsIODevice::new(
                js_read,
                js_write,
                js_handle_trigger,
                flags & JS_DEVICE_PHASE_2_WRITE != 0,
                Rc::clone(&self.ic32_latch),
            )),
            interrupt_type,
            speed,
        )
    }

    pub fn add_js_timer_device(&mut self, js_handle_trigger: Function) -> TimerDeviceID {
        self.timer_devices
            .add_device(Box::new(JsTimerDevice::new(js_handle_trigger)))
    }

    pub fn reset(&mut self) {
        let mut cycle_manager =
            CycleManager::new(self.cycles, &mut self.timer_devices, &mut self.address_map);

        self.cpu.reset(&mut cycle_manager);

        self.cycles = cycle_manager.get_cycles();
    }

    pub fn run(&mut self, until: u64) -> u64 {
        let mut cycle_manager =
            CycleManager::new(self.cycles, &mut self.timer_devices, &mut self.address_map);

        cycle_manager.repeat(until, |cycle_manager| {
            self.cpu.handle_next_instruction(cycle_manager);
        });

        self.cycles = cycle_manager.get_cycles();

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
}

const JS_DEVICE_ONE_MHZ: u8 = 0b0000_0001;
const JS_DEVICE_NMI: u8 = 0b0000_0010;
const JS_DEVICE_IRQ: u8 = 0b0000_0100;
const JS_DEVICE_PHASE_2_WRITE: u8 = 0b0001_0000;
