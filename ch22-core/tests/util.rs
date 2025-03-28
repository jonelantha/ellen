use ch22_core::bus::*;
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

pub struct CycleManagerMock {
    memory: HashMap<u16, u8>,
    pub cycles: Vec<(u16, u8, String)>,
    pub cycle_syncs: Vec<String>,
}

impl CycleManagerMock {
    pub fn new(initial_ram: &Vec<(u16, u8)>) -> CycleManagerMock {
        let mut memory = HashMap::new();

        for ram_location in initial_ram {
            memory.insert(ram_location.0, ram_location.1);
        }

        CycleManagerMock {
            memory,
            cycles: Vec::new(),
            cycle_syncs: Vec::new(),
        }
    }
}

impl Bus for CycleManagerMock {
    fn phantom_read(&mut self, address: Word) {
        self.read(address, CycleOp::None);
    }

    fn read(&mut self, address: Word, op: CycleOp) -> u8 {
        let address: u16 = address.into();

        let value = *self
            .memory
            .get(&address)
            .unwrap_or_else(|| panic!("memory not set {:x}", address));

        self.cycles.push((address, value, "read".to_owned()));

        self.cycle_syncs.push(get_sync_status_text(op));

        value
    }

    fn write(&mut self, address: Word, value: u8, op: CycleOp) {
        let address: u16 = address.into();

        self.memory.insert(address, value);

        self.cycles.push((address, value, "write".to_owned()));

        self.cycle_syncs.push(get_sync_status_text(op));
    }

    fn complete(&self) {}
}

fn get_sync_status_text(op: CycleOp) -> String {
    match op {
        CycleOp::CheckInterrupt => "sync+check_interrupt",
        CycleOp::Sync => "sync",
        CycleOp::None => "",
    }
    .to_string()
}
