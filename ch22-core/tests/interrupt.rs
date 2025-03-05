mod util;

use ch22_core::cpu::executor::*;
use ch22_core::cpu::registers::*;
use serde::Deserialize;
use std::fs;
use util::{CPUTestState, CycleManagerMock};

type CPUCycles = Vec<(u16, u8, String)>;

#[derive(Deserialize)]
struct InterruptTestParams {
    name: String,
    initial: CPUTestState,
    r#final: CPUTestState,
    cycles: CPUCycles,
}

#[test]
fn _irq_cycles_test() {
    interrupt_tests_from_file("irq", false);
}

#[test]
fn _nmi_cycles_test() {
    interrupt_tests_from_file("nmi", true);
}

fn interrupt_tests_from_file(file: &str, nmi: bool) {
    let data =
        fs::read_to_string(format!("./tests/test_cases/{file}.json")).expect("Unable to read file");

    let test_params: Vec<InterruptTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            interrupt_cycles_test(
                &test_param.name,
                &test_param.initial,
                &test_param.r#final,
                &test_param.cycles,
                nmi,
            )
        });

        if panics.is_err() {
            panic!("error was in {:?}", test_param.name)
        }
    }
}

fn interrupt_cycles_test(
    _name: &str,
    initial_state: &CPUTestState,
    final_state: &CPUTestState,
    expected_cycles: &CPUCycles,
    nmi: bool,
) {
    let mut registers = Registers {
        pc: initial_state.pc,
        s: initial_state.s,
        a: initial_state.a,
        x: initial_state.x,
        y: initial_state.y,
        p: initial_state.p.into(),
    };

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram);

    let mut executor = Executor::new(&mut cycle_manager_mock, &mut registers);

    executor.interrupt(nmi);

    assert_eq!(
        &cycle_manager_mock.cycles, expected_cycles,
        "cycles mismatch"
    );

    assert_eq!(registers.pc, final_state.pc, "pc mismatch");
    assert_eq!(registers.s, final_state.s, "s mismatch");
    assert_eq!(registers.a, final_state.a, "a mismatch");
    assert_eq!(registers.x, final_state.x, "x mismatch");
    assert_eq!(registers.y, final_state.y, "y mismatch");

    assert_eq!(
        registers.p,
        StatusRegister::from(final_state.p),
        "p mismatch"
    );
}
