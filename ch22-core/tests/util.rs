use ch22_core::cpu::interrupt_due_state::*;
use ch22_core::cpu::registers::Registers;
use ch22_core::cpu_io::*;
use ch22_core::interrupt_type::*;
use ch22_core::word::*;
use serde::Deserialize;

use std::collections::HashMap;

#[derive(Deserialize)]
pub struct CPUTestState {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub ram: Vec<(u16, u8)>,
}

impl From<&CPUTestState> for Registers {
    fn from(test_state: &CPUTestState) -> Self {
        Registers {
            program_counter: test_state.pc.into(),
            stack_pointer: test_state.s,
            accumulator: test_state.a,
            x: test_state.x,
            y: test_state.y,
            flags: test_state.p.into(),
        }
    }
}

#[allow(dead_code)]
pub type CPUCycles = Vec<(u16, u8, String)>;

#[derive(Deserialize)]
pub struct TestInterruptOnOff {
    pub on: u8,
    pub off: u8,
}

pub type TestInterruptOnOffList = Vec<TestInterruptOnOff>;

#[derive(Deserialize)]
pub struct InterruptDueTestState {
    pub previous_nmi: bool,
    pub interrupt_due: String,
}

impl From<&InterruptDueTestState> for InterruptDueState {
    fn from(test_state: &InterruptDueTestState) -> Self {
        InterruptDueState {
            previous_nmi: test_state.previous_nmi,
            interrupt_due: match test_state.interrupt_due.as_str() {
                "nmi" => Some(InterruptType::NMI),
                "irq" => Some(InterruptType::IRQ),
                _ => None,
            },
        }
    }
}

pub struct CycleManagerMock<'a> {
    memory: HashMap<u16, u8>,
    irq_on_off_list: &'a Option<TestInterruptOnOffList>,
    nmi_on_off_list: &'a Option<TestInterruptOnOffList>,
    cycle_check_nmi: bool,
    cycle_check_irq: bool,
    pub cycles: Vec<(u16, u8, String)>,
    pub cycle_syncs: Vec<String>,
}

impl<'a> CycleManagerMock<'a> {
    pub fn new(
        initial_ram: &Vec<(u16, u8)>,
        irq_on_off_list: &'a Option<TestInterruptOnOffList>,
        nmi_on_off_list: &'a Option<TestInterruptOnOffList>,
    ) -> CycleManagerMock<'a> {
        let mut memory = HashMap::new();

        for ram_location in initial_ram {
            memory.insert(ram_location.0, ram_location.1);
        }

        CycleManagerMock {
            memory,
            irq_on_off_list,
            nmi_on_off_list,
            cycle_check_nmi: false,
            cycle_check_irq: false,
            cycles: Vec::new(),
            cycle_syncs: Vec::new(),
        }
    }
}

impl CpuIO for CycleManagerMock<'_> {
    fn phantom_read(&mut self, address: Word) {
        self.read(address);
    }

    fn read(&mut self, address: Word) -> u8 {
        let address: u16 = address.into();

        let value = *self
            .memory
            .get(&address)
            .unwrap_or_else(|| panic!("memory not set {:x}", address));

        self.cycles.push((address, value, "read".to_owned()));

        self.cycle_syncs.push(get_sync_status_text(
            self.cycle_check_nmi,
            self.cycle_check_irq,
        ));

        self.cycle_check_nmi = false;

        value
    }

    fn write(&mut self, address: Word, value: u8) {
        let address: u16 = address.into();

        self.memory.insert(address, value);

        self.cycles.push((address, value, "write".to_owned()));

        self.cycle_syncs.push(get_sync_status_text(
            self.cycle_check_nmi,
            self.cycle_check_irq,
        ));

        self.cycle_check_nmi = false;
    }

    fn get_interrupt(&mut self, interrupt_type: InterruptType) -> bool {
        match interrupt_type {
            InterruptType::IRQ => self.cycle_check_irq = true,
            InterruptType::NMI => self.cycle_check_nmi = true,
        }

        let current_cycle = self.cycles.len() as u8;

        match interrupt_type {
            InterruptType::IRQ => is_in_on_off_range(self.irq_on_off_list, current_cycle),
            InterruptType::NMI => is_in_on_off_range(self.nmi_on_off_list, current_cycle),
        }
    }
}

fn is_in_on_off_range(on_off_list: &Option<TestInterruptOnOffList>, cycle: u8) -> bool {
    if let Some(list) = on_off_list {
        list.iter()
            .any(|range| cycle >= range.on && cycle < range.off)
    } else {
        false
    }
}

fn get_sync_status_text(check_nmi: bool, check_irq: bool) -> String {
    match (check_nmi, check_irq) {
        (true, true) => "check_nmi+irq",
        (true, false) => "check_nmi",
        _ => "",
    }
    .to_string()
}
