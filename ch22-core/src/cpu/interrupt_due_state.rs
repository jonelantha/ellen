use crate::cpu_io::*;

#[derive(PartialEq, Default, Debug)]
pub struct InterruptDueState {
    pub previous_nmi: bool,
    pub interrupt_due: Option<InterruptType>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InterruptType {
    IRQ,
    NMI,
}

pub fn update_interrupt_due_state<IO: CpuIO>(
    interrupt_due_state: &mut InterruptDueState,
    io: &mut IO,
    interrupt_disable: bool,
) {
    let nmi = io.get_nmi();

    if interrupt_due_state.previous_nmi != nmi {
        if nmi {
            interrupt_due_state.interrupt_due = Some(InterruptType::NMI);
        }
        interrupt_due_state.previous_nmi = nmi;
    }

    if !interrupt_disable && interrupt_due_state.interrupt_due.is_none() {
        if io.get_irq() {
            interrupt_due_state.interrupt_due = Some(InterruptType::IRQ);
        }
    }
}
