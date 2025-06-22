use crate::cpu_io::*;
use crate::device_map::*;
use crate::word::Word;

const CYCLE_WRAP: u32 = 0x3FFFFFFF;

pub struct CycleManager {
    pub cycles: u32,
    needs_phase_2: Option<Word>,
    get_irq_nmi: Box<dyn Fn(u32) -> u64>,
    wrap_counts: Box<dyn Fn(u32)>,
    pub device_map: DeviceMap,
}

impl CycleManager {
    pub fn new(
        device_map: DeviceMap,
        get_irq_nmi: Box<dyn Fn(u32) -> u64>,
        wrap_counts: Box<dyn Fn(u32)>,
    ) -> Self {
        CycleManager {
            cycles: 0,
            get_irq_nmi,
            wrap_counts,
            needs_phase_2: None,
            device_map,
        }
    }
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        self.device_map
            .get_device(address)
            .read(address, &mut self.cycles)
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        let needs_phase_2 =
            self.device_map
                .get_device(address)
                .write(address, value, &mut self.cycles);

        if needs_phase_2 {
            self.needs_phase_2 = Some(address);
        }
    }

    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool) {
        let interrupt_flags = self.device_map.io_space.get_interrupt(self.cycles);

        let old_interrupt_flags = (self.get_irq_nmi)(self.cycles);

        let nmi = (interrupt_flags & 0x0f) != 0;
        let irq = (interrupt_flags & 0xf0) != 0;

        let old_irq = (old_interrupt_flags & 0xf0) != 0;

        ((irq | old_irq) & !interrupt_disable, nmi)
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        if let Some(address) = self.needs_phase_2 {
            let device = self.device_map.get_device(address);

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
