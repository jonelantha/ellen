use crate::cpu_io::*;
use crate::device::device::*;
use crate::device::io_space::*;
use crate::device::paged_rom::*;
use crate::device::ram::*;
use crate::device::rom::Ch22Rom;
use crate::word::Word;

const CYCLE_WRAP: u32 = 0x3FFFFFFF;

pub struct CycleManager {
    pub cycles: u32,
    needs_phase_2: Option<u16>,
    get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
    wrap_counts: Box<dyn Fn(u32)>,
    pub device_list: DeviceList,
}

impl CycleManager {
    pub fn new(
        ram: Ch22Ram,
        paged_rom: Ch22PagedRom,
        io_space: Ch22IOSpace,
        rom: Ch22Rom,
        get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
        wrap_counts: Box<dyn Fn(u32)>,
    ) -> Self {
        CycleManager {
            cycles: 0,
            get_irq_nmi,
            wrap_counts,
            needs_phase_2: None,
            device_list: DeviceList::new(ram, paged_rom, io_space, rom),
        }
    }
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        let address: u16 = address.into();

        let device = self.device_list.get_device(address);

        let is_slow = device.is_slow(address);

        if is_slow && self.cycles & 1 != 0 {
            self.cycles += 1;
        }

        let value = device.read(address, self.cycles);

        if is_slow {
            self.cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        let address: u16 = address.into();

        let device = self.device_list.get_device(address);

        let is_slow = device.is_slow(address);

        if is_slow && self.cycles & 1 != 0 {
            self.cycles += 1;
        }

        if device.write(address, value, self.cycles) {
            self.needs_phase_2 = Some(address);
        }

        if is_slow {
            self.cycles += 1;
        }
    }

    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool) {
        let (irq, nmi) = (self.get_irq_nmi)(self.cycles);

        (irq & !interrupt_disable, nmi)
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        if let Some(address) = self.needs_phase_2 {
            let device = self.device_list.get_device(address);

            device.phase_2(address, self.cycles);

            self.needs_phase_2 = None;
        }

        self.cycles += 1;

        if self.cycles > CYCLE_WRAP {
            (self.wrap_counts)(CYCLE_WRAP);
            self.cycles -= CYCLE_WRAP;
        }
    }
}

pub struct DeviceList {
    pub ram: Ch22Ram,
    pub paged_rom: Ch22PagedRom,
    io_space: Ch22IOSpace,
    pub rom: Ch22Rom,
}

impl DeviceList {
    pub fn new(ram: Ch22Ram, paged_rom: Ch22PagedRom, io_space: Ch22IOSpace, rom: Ch22Rom) -> Self {
        DeviceList {
            ram,
            paged_rom,
            io_space,
            rom,
        }
    }

    fn get_device(&mut self, address: u16) -> &mut dyn Ch22Device {
        match address {
            ..0x8000 => &mut self.ram,
            0x8000..0xc000 => &mut self.paged_rom,
            0xc000..0xfc00 => &mut self.rom,
            0xfc00..0xff00 => &mut self.io_space,
            0xff00.. => &mut self.rom,
        }
    }
}
