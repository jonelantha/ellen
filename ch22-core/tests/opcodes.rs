use ch22_core::cpu::*;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[allow(dead_code)]
#[derive(Deserialize)]
struct CPUTestState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

type CPUCycles = Vec<(u16, u8, String)>;

#[derive(Deserialize)]
struct CPUTestParams {
    name: String,
    initial: CPUTestState,
    r#final: CPUTestState,
    cycles: CPUCycles,
}

#[test]
fn _8d_test() {
    opcode_tests_from_file("8d");
}

#[test]
fn _a9_test() {
    opcode_tests_from_file("a9");
}

fn opcode_tests_from_file(opcode: &str) {
    let data = fs::read_to_string(format!(
        "./tests/single_step_tests_65x02/6502/v1/{opcode}.json"
    ))
    .expect("Unable to read file");

    let test_params: Vec<CPUTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            opcode_test(
                &test_param.name,
                &test_param.initial,
                &test_param.r#final,
                &test_param.cycles,
            )
        });

        if panics.is_err() {
            panic!("error was in {:?}", test_param.name)
        }
    }
}

fn opcode_test(
    _name: &str,
    initial_state: &CPUTestState,
    final_state: &CPUTestState,
    expected_cycles: &CPUCycles,
) {
    let mut cpu_state = Ch22CpuState::new();
    cpu_state.pc = initial_state.pc;
    cpu_state.a = initial_state.a;
    cpu_state.set_p(initial_state.p);

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram);

    cpu_state.handle_next_instruction(&mut cycle_manager_mock);

    assert_eq!(&cycle_manager_mock.cycles, expected_cycles);

    assert_eq!(cpu_state.pc, final_state.pc);
    assert_eq!(cpu_state.a, final_state.a);
    assert_eq!(cpu_state.get_p(), final_state.p);
}

struct CycleManagerMock {
    memory: HashMap<u16, u8>,
    cycles: Vec<(u16, u8, String)>,
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
        }
    }
}

impl CycleManagerTrait for CycleManagerMock {
    fn read(&mut self, address: u16, _sync: bool, _check_interrupt: bool) -> u8 {
        let value = *self
            .memory
            .get(&address)
            .expect(&format!("memory not set {:x}", address));

        self.cycles.push((address, value, "read".to_owned()));

        value
    }

    fn write(&mut self, address: u16, value: u8, _sync: bool, _check_interrupt: bool) {
        self.memory.insert(address, value);

        self.cycles.push((address, value, "write".to_owned()));
    }

    fn complete(&self) {}
}
