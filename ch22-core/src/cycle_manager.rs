use crate::cpu_io::*;
use crate::memory::*;
use crate::word::Word;

pub struct CycleManager<'a> {
    machine_cycles: &'a mut u32,
    needs_phase_2: bool,
    memory: &'a mut Ch22Memory,
    get_irq_nmi: &'a (dyn Fn(u32) -> (bool, bool) + 'a),
    do_phase_2: &'a (dyn Fn(u32) + 'a),
    is_first: bool,
}

impl<'a> CycleManager<'a> {
    pub fn new(
        memory: &'a mut Ch22Memory,
        machine_cycles: &'a mut u32,
        get_irq_nmi: &'a (dyn Fn(u32) -> (bool, bool) + 'a),
        do_phase_2: &'a (dyn Fn(u32) + 'a),
    ) -> Self {
        CycleManager {
            machine_cycles,
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

        let is_io = self.memory.is_io(address.into());

        if is_io && *self.machine_cycles & 1 != 0 {
            *self.machine_cycles += 1;
        }

        let value = self.memory.read(address.into(), *self.machine_cycles);

        if is_io {
            *self.machine_cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8) {
        self.cycle_end();

        let is_io = self.memory.is_io(address.into());

        if is_io && *self.machine_cycles & 1 != 0 {
            *self.machine_cycles += 1;
        }

        self.needs_phase_2 = self
            .memory
            .write(address.into(), value, *self.machine_cycles);

        if is_io {
            *self.machine_cycles += 1;
        }
    }

    fn complete(&mut self) {
        self.cycle_end();
    }

    fn get_irq_nmi(&mut self, interrupt_disable: bool) -> (bool, bool) {
        let (irq, nmi) = (self.get_irq_nmi)(*self.machine_cycles);

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
            (self.do_phase_2)(*self.machine_cycles);
            self.needs_phase_2 = false;
        }

        *self.machine_cycles += 1;
    }
}
