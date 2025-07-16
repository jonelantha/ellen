use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::io_device::*;
use crate::word::Word;

#[derive(Default)]
pub struct IODeviceMock {
    memory: HashMap<u16, u8>,
    interrupt_on: bool,
    accesses: Rc<RefCell<IODeviceAccesses>>,
}

impl IODeviceMock {
    pub fn new(initial_ram: &[(u16, u8)], interrupt_on: bool) -> Self {
        let mut memory = HashMap::new();

        for ram_location in initial_ram {
            memory.insert(ram_location.0, ram_location.1);
        }

        IODeviceMock {
            memory,
            interrupt_on,
            ..IODeviceMock::default()
        }
    }

    pub fn get_accesses(&self) -> Rc<RefCell<IODeviceAccesses>> {
        self.accesses.clone()
    }

    fn push_memory_access(&mut self, access: MemoryAccess) {
        self.accesses.borrow_mut().memory.push(access);
    }
}

impl IODevice for IODeviceMock {
    fn read(&mut self, address: Word, cycles: u64) -> u8 {
        let address: u16 = address.into();

        self.push_memory_access(MemoryAccess::Read(address, cycles));

        *self
            .memory
            .get(&address)
            .unwrap_or_else(|| panic!("memory not set {address:x}"))
    }

    fn write(&mut self, address: Word, value: u8, cycles: u64) -> bool {
        let address: u16 = address.into();

        self.memory.insert(address, value);

        self.push_memory_access(MemoryAccess::Write(address, value, cycles));

        false
    }

    fn get_interrupt(&mut self, cycles: u64) -> bool {
        self.accesses.borrow_mut().interrupt.push(cycles);

        self.interrupt_on
    }
}

#[derive(Debug, PartialEq)]
pub enum MemoryAccess {
    Read(u16, u64),
    Write(u16, u8, u64),
}

#[derive(Default)]
pub struct IODeviceAccesses {
    pub memory: Vec<MemoryAccess>,
    pub interrupt: Vec<u64>,
}
