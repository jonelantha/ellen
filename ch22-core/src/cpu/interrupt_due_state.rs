use crate::cpu_io::CpuIO;

#[derive(PartialEq, Default, Debug)]
pub struct InterruptDueState {
    pub previous_nmi: bool,
    pub interrupt_due: InterruptType,
}

#[derive(PartialEq, Default, Clone, Copy, Debug)]
pub enum InterruptType {
    #[default]
    None,
    IRQ,
    NMI,
}

pub fn update_interrupt_due_state<IO: CpuIO>(
    interrupt_due_state: &mut InterruptDueState,
    io: &mut IO,
    interrupt_disable: bool,
) {
    let (irq, nmi) = io.get_irq_nmi(interrupt_disable);

    if interrupt_due_state.previous_nmi != nmi {
        if nmi {
            interrupt_due_state.interrupt_due = InterruptType::NMI;
        }
        interrupt_due_state.previous_nmi = nmi;
    }

    if irq && interrupt_due_state.interrupt_due == InterruptType::None {
        interrupt_due_state.interrupt_due = InterruptType::IRQ;
    }
}
