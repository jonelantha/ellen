mod common;

use ch22_core::cpu::executor::*;
use ch22_core::cpu::interrupt_due_state::*;
use ch22_core::cpu::registers::*;
use serde::Deserialize;
use std::fs;

use crate::common::cycle_manager_mock::CycleManagerMock;
use crate::common::json_data::*;

#[derive(Deserialize)]
struct InterruptTestParams {
    name: String,
    initial: CPUTestState,
    initial_interrupt_due_state: InterruptDueTestState,
    irq: Option<TestInterruptOnOffList>,
    nmi: Option<TestInterruptOnOffList>,
    r#final: CPUTestState,
    final_interrupt_due_state: InterruptDueTestState,
    cycles: Option<CPUCycles>,
}

#[test]
fn interrupt_tests_from_file() {
    let data =
        fs::read_to_string("./tests/test_cases/interrupt.json").expect("Unable to read file");

    let test_params: Vec<InterruptTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            interrupt_cycles_test(
                &test_param.initial,
                &test_param.initial_interrupt_due_state,
                &test_param.irq,
                &test_param.nmi,
                &test_param.r#final,
                &test_param.final_interrupt_due_state,
                &test_param.cycles,
            )
        });

        if panics.is_err() {
            panic!("error was in {:?}", test_param.name)
        }
    }
}

fn interrupt_cycles_test(
    initial_state: &CPUTestState,
    initial_interrupt_due_state: &InterruptDueTestState,
    irq_on_off: &Option<TestInterruptOnOffList>,
    nmi_on_off: &Option<TestInterruptOnOffList>,
    final_state: &CPUTestState,
    final_interrupt_due_state: &InterruptDueTestState,
    expected_cycles: &Option<CPUCycles>,
) {
    let mut registers: Registers = initial_state.into();

    let mut interrupt_due_state: InterruptDueState = initial_interrupt_due_state.into();

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram, irq_on_off, nmi_on_off);

    execute(
        &mut cycle_manager_mock,
        &mut registers,
        &mut interrupt_due_state,
        true,
    );

    if let Some(expected_cycles) = expected_cycles {
        assert_eq!(
            &cycle_manager_mock.cycles, expected_cycles,
            "cycles mismatch"
        );
    }

    assert_eq!(
        u16::from(registers.program_counter),
        final_state.pc,
        "pc mismatch"
    );
    assert_eq!(registers.stack_pointer, final_state.s, "s mismatch");
    assert_eq!(registers.accumulator, final_state.a, "a mismatch");
    assert_eq!(registers.x, final_state.x, "x mismatch");
    assert_eq!(registers.y, final_state.y, "y mismatch");

    assert_eq!(
        registers.flags,
        ProcessorFlags::from(final_state.p),
        "p mismatch"
    );

    assert_eq!(
        interrupt_due_state,
        InterruptDueState::from(final_interrupt_due_state),
        "interrupt_due_state mismatch"
    );
}
