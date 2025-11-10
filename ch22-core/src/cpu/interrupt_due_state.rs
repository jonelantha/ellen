use super::cpu_io::CpuIO;
use super::interrupt_type::InterruptType;

#[derive(PartialEq, Default, Debug)]
pub struct InterruptDueState {
    pub previous_nmi: bool,
    pub interrupt_due: Option<InterruptType>,
}

impl InterruptDueState {
    pub fn update<IO: CpuIO>(&mut self, io: &mut IO, interrupt_disable: bool) {
        let nmi = io.get_interrupt(InterruptType::NMI);

        if self.previous_nmi != nmi {
            if nmi {
                self.interrupt_due = Some(InterruptType::NMI);
            }
            self.previous_nmi = nmi;
        }

        if !interrupt_disable
            && self.interrupt_due.is_none()
            && io.get_interrupt(InterruptType::IRQ)
        {
            self.interrupt_due = Some(InterruptType::IRQ);
        }
    }
}
