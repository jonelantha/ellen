use ch22_core::cpu::interrupt_due_state::*;
use ch22_core::cpu::registers::*;
use ch22_core::interrupt_type::*;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct TestInterruptOnOff {
    pub on: u8,
    pub off: u8,
}

pub type TestInterruptOnOffList = Vec<TestInterruptOnOff>;

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

impl From<&CPUTestState> for Registers {
    fn from(test_state: &CPUTestState) -> Self {
        Registers {
            program_counter: test_state.pc.into(),
            stack_pointer: test_state.s,
            accumulator: test_state.a,
            x: test_state.x,
            y: test_state.y,
            flags: test_state.p.into(),
        }
    }
}

#[allow(dead_code)]
pub type CPUCycles = Vec<(u16, u8, String)>;

#[derive(Deserialize)]
pub struct InterruptDueTestState {
    pub previous_nmi: bool,
    pub interrupt_due: String,
}

impl From<&InterruptDueTestState> for InterruptDueState {
    fn from(test_state: &InterruptDueTestState) -> Self {
        InterruptDueState {
            previous_nmi: test_state.previous_nmi,
            interrupt_due: match test_state.interrupt_due.as_str() {
                "nmi" => Some(InterruptType::NMI),
                "irq" => Some(InterruptType::IRQ),
                _ => None,
            },
        }
    }
}
