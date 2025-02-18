mod util;

use ch22_core::cpu::*;
use serde::Deserialize;
use std::fs;
use util::{CPUTestState, CycleManagerMock};

#[derive(Deserialize)]
struct CycleTestParams {
    name: String,
    initial: CPUTestState,
    cycle_syncs: Vec<String>,
}

#[test]
fn opcode_cycle_tests_from_file() {
    let data = fs::read_to_string("./tests/opcode_data/cycles.json").expect("Unable to read file");

    let test_params: Vec<CycleTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            opcode_cycle_test(
                &test_param.name,
                &test_param.initial,
                &test_param.cycle_syncs,
            )
        });

        if panics.is_err() {
            panic!("error was in {:?}", test_param.name)
        }
    }
}

fn opcode_cycle_test(_name: &str, initial_state: &CPUTestState, cycle_syncs: &Vec<String>) {
    let mut cpu_state = Ch22CpuState::new();
    cpu_state.pc = initial_state.pc;
    cpu_state.s = initial_state.s;
    cpu_state.a = initial_state.a;
    cpu_state.x = initial_state.x;
    cpu_state.y = initial_state.y;
    cpu_state.set_p(initial_state.p);

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram);

    cpu_state.handle_next_instruction(&mut cycle_manager_mock);

    assert_eq!(&cycle_manager_mock.cycle_syncs, cycle_syncs);
}
