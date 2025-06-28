use crate::cpu_io::*;
use crate::interrupt_type::*;

#[derive(PartialEq, Default, Debug)]
pub struct InterruptDueState {
    pub previous_nmi: bool,
    pub interrupt_due: Option<InterruptType>,
}

pub fn update_interrupt_due_state<IO: CpuIO>(
    interrupt_due_state: &mut InterruptDueState,
    io: &mut IO,
    interrupt_disable: bool,
) {
    let nmi = io.get_interrupt(InterruptType::NMI);

    if interrupt_due_state.previous_nmi != nmi {
        if nmi {
            interrupt_due_state.interrupt_due = Some(InterruptType::NMI);
        }
        interrupt_due_state.previous_nmi = nmi;
    }

    if !interrupt_disable && interrupt_due_state.interrupt_due.is_none() {
        if io.get_interrupt(InterruptType::IRQ) {
            interrupt_due_state.interrupt_due = Some(InterruptType::IRQ);
        }
    }
}
