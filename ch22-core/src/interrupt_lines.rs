#[derive(Default)]
pub struct InterruptLines {
    pub nmi: bool,
    pub irq: bool,
}
