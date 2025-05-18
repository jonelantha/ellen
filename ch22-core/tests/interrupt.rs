mod util;

use ch22_core::cpu::executor::*;
use ch22_core::cpu::interrupt_state::*;
use ch22_core::cpu::registers::*;
use serde::Deserialize;
use std::fs;
use util::*;

#[derive(Deserialize)]
struct InterruptTestParams {
    name: String,
    initial: CPUTestState,
    initial_interrupt_state: InterruptTestState,
    irq: Option<TestInterruptOnOffList>,
    nmi: Option<TestInterruptOnOffList>,
    r#final: CPUTestState,
    final_interrupt_state: InterruptTestState,
    cycles: Option<CPUCycles>,
}

#[test]
fn interrupt_tests_from_file() {
    let data = fs::read_to_string(format!("./tests/test_cases/interrupt.json"))
        .expect("Unable to read file");

    let test_params: Vec<InterruptTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            interrupt_cycles_test(
                &test_param.name,
                &test_param.initial,
                &test_param.initial_interrupt_state,
                &test_param.irq,
                &test_param.nmi,
                &test_param.r#final,
                &test_param.final_interrupt_state,
                &test_param.cycles,
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
    initial_interrupt_state: &InterruptTestState,
    irq_on_off: &Option<TestInterruptOnOffList>,
    nmi_on_off: &Option<TestInterruptOnOffList>,
    final_state: &CPUTestState,
    final_interrupt_state: &InterruptTestState,
    expected_cycles: &Option<CPUCycles>,
) {
    let mut registers: Registers = initial_state.into();

    let mut interrupt_state: InterruptState = initial_interrupt_state.into();

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram, irq_on_off, nmi_on_off);

    execute(
        &mut cycle_manager_mock,
        &mut registers,
        &mut interrupt_state,
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
        interrupt_state,
        InterruptState::from(final_interrupt_state),
        "interrupt_state mismatch"
    );
}
