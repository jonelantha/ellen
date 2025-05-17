use crate::cpu_io::CpuIO;

#[derive(PartialEq, Default, Debug)]
pub struct InterruptState {
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

pub fn update_interrupt_state<IO: CpuIO>(
    interrupt_state: &mut InterruptState,
    io: &mut IO,
    interrupt_disable: bool,
) {
    let (irq, nmi) = io.get_irq_nmi(interrupt_disable);

    if interrupt_state.previous_nmi != nmi {
        if nmi {
            interrupt_state.interrupt_due = InterruptType::NMI;
        }
        interrupt_state.previous_nmi = nmi;
    }

    if irq && interrupt_state.interrupt_due == InterruptType::None {
        interrupt_state.interrupt_due = InterruptType::IRQ;
    }
}
