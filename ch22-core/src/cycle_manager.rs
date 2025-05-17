use crate::cpu_io::*;
use crate::memory::*;
use crate::word::Word;

pub struct CycleManager<'a> {
    pub cycles: u8,
    needs_phase_2: bool,
    memory: &'a mut Ch22Memory,
    get_irq_nmi: &'a (dyn Fn(u8) -> (bool, bool) + 'a),
    do_phase_2: &'a (dyn Fn(u8) + 'a),
    is_first: bool,
}

impl<'a> CycleManager<'a> {
    pub fn new(
        memory: &'a mut Ch22Memory,
        get_irq_nmi: &'a (dyn Fn(u8) -> (bool, bool) + 'a),
        do_phase_2: &'a (dyn Fn(u8) + 'a),
    ) -> Self {
        CycleManager {
            cycles: 0,
            memory,
            get_irq_nmi,
            do_phase_2,
            needs_phase_2: false,
            is_first: true,
        }
    }
}

impl CpuIO for CycleManager<'_> {
    fn phantom_read(&mut self, _address: Word) {
        self.cycle_end();
    }

    fn read(&mut self, address: Word) -> u8 {
        self.cycle_end();

        self.memory.read(address.into(), self.cycles)
    }

    fn write(&mut self, address: Word, value: u8) {
        self.cycle_end();

        self.needs_phase_2 = self.memory.write(address.into(), value, self.cycles);
    }

    fn complete(&mut self) {
        self.cycle_end();
    }

    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool) {
        let (irq, nmi) = (self.get_irq_nmi)(self.cycles);

        (irq & !interrupt_disable, nmi)
    }
}

impl CycleManager<'_> {
    fn cycle_end(&mut self) {
        if self.is_first {
            self.is_first = false;
            return;
        };

        if self.needs_phase_2 {
            (self.do_phase_2)(self.cycles);
            self.needs_phase_2 = false;
        }

        self.cycles += 1;
    }
}
