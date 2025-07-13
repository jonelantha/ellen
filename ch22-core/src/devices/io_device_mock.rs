use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::io_device::*;
use crate::word::Word;

#[derive(Default)]
pub struct IODeviceMock {
    memory: HashMap<u16, u8>,
    accesses: Rc<RefCell<Vec<Access>>>,
}

impl IODeviceMock {
    pub fn new(initial_ram: &[(u16, u8)]) -> Self {
        let mut memory = HashMap::new();

        for ram_location in initial_ram {
            memory.insert(ram_location.0, ram_location.1);
        }

        IODeviceMock {
            memory,
            ..IODeviceMock::default()
        }
    }

    pub fn get_accesses(&self) -> Rc<RefCell<Vec<Access>>> {
        self.accesses.clone()
    }
}

impl IODevice for IODeviceMock {
    fn read(&mut self, address: Word, cycles: u64) -> u8 {
        let address: u16 = address.into();

        self.accesses
            .borrow_mut()
            .push(Access::Read(address, cycles));

        *self
            .memory
            .get(&address)
            .unwrap_or_else(|| panic!("memory not set {address:x}"))
    }

    fn write(&mut self, address: Word, value: u8, cycles: u64) -> bool {
        let address: u16 = address.into();

        self.memory.insert(address, value);

        self.accesses
            .borrow_mut()
            .push(Access::Write(address, value, cycles));

        false
    }
}

#[derive(Debug, PartialEq)]
pub enum Access {
    Read(u16, u64),
    Write(u16, u8, u64),
}
