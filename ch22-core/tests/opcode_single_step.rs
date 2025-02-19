mod util;

use ch22_core::cpu::*;
use serde::Deserialize;
use std::fs;
use util::{CPUTestState, CycleManagerMock};

type CPUCycles = Vec<(u16, u8, String)>;

#[derive(Deserialize)]
struct SingleStepTestParams {
    name: String,
    initial: CPUTestState,
    r#final: CPUTestState,
    cycles: CPUCycles,
}

#[test]
fn _0a_test() {
    opcode_single_step_tests_from_file("0a");
}

#[test]
fn _10_test() {
    opcode_single_step_tests_from_file("10");
}

#[test]
fn _48_test() {
    opcode_single_step_tests_from_file("48");
}

#[test]
fn _78_test() {
    opcode_single_step_tests_from_file("78");
}

#[test]
fn _85_test() {
    opcode_single_step_tests_from_file("85");
}

#[test]
fn _86_test() {
    opcode_single_step_tests_from_file("86");
}

#[test]
fn _8d_test() {
    opcode_single_step_tests_from_file("8d");
}

#[test]
fn _8e_test() {
    opcode_single_step_tests_from_file("8e");
}

#[test]
fn _91_test() {
    opcode_single_step_tests_from_file("91");
}

#[test]
fn _9a_test() {
    opcode_single_step_tests_from_file("9a");
}

#[test]
fn _a2_test() {
    opcode_single_step_tests_from_file("a2");
}

#[test]
fn _a8_test() {
    opcode_single_step_tests_from_file("a8");
}

#[test]
fn _a9_test() {
    opcode_single_step_tests_from_file("a9");
}

#[test]
fn _ad_test() {
    opcode_single_step_tests_from_file("ad");
}

#[test]
fn _c5_test() {
    opcode_single_step_tests_from_file("c5");
}

#[test]
fn _c8_test() {
    opcode_single_step_tests_from_file("c8");
}

#[test]
fn _ca_test() {
    opcode_single_step_tests_from_file("ca");
}

#[test]
fn _e6_test() {
    opcode_single_step_tests_from_file("e6");
}

#[test]
fn _e8_test() {
    opcode_single_step_tests_from_file("e8");
}

#[test]
fn _d0_test() {
    opcode_single_step_tests_from_file("d0");
}

#[test]
fn _d8_test() {
    opcode_single_step_tests_from_file("d8");
}

#[test]
fn _f0_test() {
    opcode_single_step_tests_from_file("f0");
}

fn opcode_single_step_tests_from_file(opcode: &str) {
    let data = fs::read_to_string(format!(
        "./tests/single_step_tests_65x02/6502/v1/{opcode}.json"
    ))
    .expect("Unable to read file");

    let test_params: Vec<SingleStepTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            opcode_single_step_test(
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

fn opcode_single_step_test(
    _name: &str,
    initial_state: &CPUTestState,
    final_state: &CPUTestState,
    expected_cycles: &CPUCycles,
) {
    let mut cpu_state = Ch22CpuState::new();
    cpu_state.pc = initial_state.pc;
    cpu_state.s = initial_state.s;
    cpu_state.a = initial_state.a;
    cpu_state.x = initial_state.x;
    cpu_state.y = initial_state.y;
    cpu_state.set_p(initial_state.p);

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram);

    cpu_state.handle_next_instruction(&mut cycle_manager_mock);

    assert_eq!(&cycle_manager_mock.cycles, expected_cycles);

    assert_eq!(cpu_state.pc, final_state.pc);
    assert_eq!(cpu_state.s, final_state.s);
    assert_eq!(cpu_state.a, final_state.a);
    assert_eq!(cpu_state.x, final_state.x);
    assert_eq!(cpu_state.y, final_state.y);
    assert_eq!(cpu_state.get_p(), final_state.p);
}
