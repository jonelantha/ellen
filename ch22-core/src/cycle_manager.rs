use crate::cpu_io::*;
use crate::memory::*;
use crate::word::Word;

const CYCLE_WRAP: u32 = 0x3FFFFFFF;

pub struct CycleManager {
    pub machine_cycles: u32,
    needs_phase_2: bool,
    pub memory: Ch22Memory,
    get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
    do_phase_2: Box<dyn Fn(u32)>,
    wrap_counts: Box<dyn Fn(u32)>,
}

impl CycleManager {
    pub fn new(
        memory: Ch22Memory,
        get_irq_nmi: Box<dyn Fn(u32) -> (bool, bool)>,
        do_phase_2: Box<dyn Fn(u32)>,
        wrap_counts: Box<dyn Fn(u32)>,
    ) -> Self {
        CycleManager {
            machine_cycles: 0,
            memory,
            get_irq_nmi,
            do_phase_2,
            wrap_counts,
            needs_phase_2: false,
        }
    }
}

impl CpuIO for CycleManager {
    fn phantom_read(&mut self, _address: Word) {
        self.end_previous_cycle();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.end_previous_cycle();

        let is_io = self.memory.is_io(address.into());

        if is_io && self.machine_cycles & 1 != 0 {
            self.machine_cycles += 1;
        }

        let value = self.memory.read(address.into(), self.machine_cycles);

        if is_io {
            self.machine_cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8) {
        self.end_previous_cycle();

        let is_io = self.memory.is_io(address.into());

        if is_io && self.machine_cycles & 1 != 0 {
            self.machine_cycles += 1;
        }

        self.needs_phase_2 = self
            .memory
            .write(address.into(), value, self.machine_cycles);

        if is_io {
            self.machine_cycles += 1;
        }
    }

    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool) {
        let (irq, nmi) = (self.get_irq_nmi)(self.machine_cycles);

        (irq & !interrupt_disable, nmi)
    }
}

impl CycleManager {
    fn end_previous_cycle(&mut self) {
        if self.needs_phase_2 {
            (self.do_phase_2)(self.machine_cycles);
            self.needs_phase_2 = false;
        }

        self.machine_cycles += 1;

        if self.machine_cycles > CYCLE_WRAP {
            (self.wrap_counts)(CYCLE_WRAP);
            self.machine_cycles -= CYCLE_WRAP;
        }
    }
}
