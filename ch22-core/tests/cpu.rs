use ch22_core::cpu::*;
use rstest::*;
use rstest_reuse::{self, *};
use std::collections::HashMap;

pub mod cpu_cases;

#[allow(dead_code)]
struct Initial {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

#[allow(dead_code)]
struct Expected<'a> {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    cycles: Vec<(u16, u8, &'a str)>,
}

use cpu_cases::x8d::*;

#[apply(x8d_cases)]
fn x8d_test(#[case] initial: Initial, #[case] expected: Expected) {
    opcode_test(initial, expected)
}

use cpu_cases::xa9::*;

#[apply(xa9_cases)]
fn xa9_test(#[case] initial: Initial, #[case] expected: Expected) {
    opcode_test(initial, expected)
}

fn opcode_test(initial: Initial, expected: Expected) {
    let mut cpu_state = Ch22CpuState::new();
    cpu_state.pc = initial.pc;
    cpu_state.a = initial.a;
    cpu_state.set_p(initial.p);

    let mut cycle_manager_mock = CycleManagerMock::new(initial.ram);

    cpu_state.handle_next_instruction(&mut cycle_manager_mock);

    assert_eq!(cycle_manager_mock.cycles, expected.cycles);

    assert_eq!(cpu_state.pc, expected.pc);
    assert_eq!(cpu_state.a, expected.a);
    assert_eq!(cpu_state.get_p(), expected.p);
}

struct CycleManagerMock {
    memory: HashMap<u16, u8>,
    cycles: Vec<(u16, u8, &'static str)>,
}

impl CycleManagerMock {
    pub fn new(initial_ram: Vec<(u16, u8)>) -> CycleManagerMock {
        let mut memory = HashMap::new();

        for ram_location in &initial_ram {
            memory.insert(ram_location.0, ram_location.1);
        }

        CycleManagerMock {
            memory,
            cycles: Vec::new(),
        }
    }
}

impl CycleManagerTrait for CycleManagerMock {
    fn read(&mut self, address: u16, _sync: bool, _check_interrupt: bool) -> u8 {
        let value = *self
            .memory
            .get(&address)
            .expect(&format!("memory not set {:?}", address));

        self.cycles.push((address, value, "read"));

        value
    }

    fn write(&mut self, address: u16, value: u8, _sync: bool, _check_interrupt: bool) {
        self.memory.insert(address, value);

        self.cycles.push((address, value, "write"));
    }

    fn complete(&self) {}
}
