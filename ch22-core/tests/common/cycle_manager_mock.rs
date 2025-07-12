use ch22_core::cpu::cpu_io::*;
use ch22_core::interrupt_type::*;
use ch22_core::word::*;

use std::collections::HashMap;

use super::json_data::TestInterruptOnOffList;

#[derive(Default)]
pub struct CycleManagerMock {
    memory: HashMap<u16, u8>,
    irq_on_off_list: Option<TestInterruptOnOffList>,
    nmi_on_off_list: Option<TestInterruptOnOffList>,
    cycle_check_nmi: bool,
    cycle_check_irq: bool,
    pub cycles: Vec<(u16, u8, String)>,
    pub cycle_syncs: Vec<String>,
}

impl CycleManagerMock {
    pub fn new(
        initial_ram: &Vec<(u16, u8)>,
        irq_on_off_list: Option<TestInterruptOnOffList>,
        nmi_on_off_list: Option<TestInterruptOnOffList>,
    ) -> CycleManagerMock {
        let mut memory = HashMap::new();

        for ram_location in initial_ram {
            memory.insert(ram_location.0, ram_location.1);
        }

        CycleManagerMock {
            memory,
            irq_on_off_list,
            nmi_on_off_list,
            ..Default::default()
        }
    }
}

impl CpuIO for CycleManagerMock {
    fn phantom_read(&mut self, address: Word) {
        self.read(address);
    }

    fn read(&mut self, address: Word) -> u8 {
        let address: u16 = address.into();

        let value = *self
            .memory
            .get(&address)
            .unwrap_or_else(|| panic!("memory not set {address:x}"));

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
            InterruptType::IRQ => is_in_on_off_range(&self.irq_on_off_list, current_cycle),
            InterruptType::NMI => is_in_on_off_range(&self.nmi_on_off_list, current_cycle),
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
