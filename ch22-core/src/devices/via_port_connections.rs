use std::{cell::RefCell, rc::Rc};

/// What is wired to a VIA's ports. The VIA reports the state of both ports
/// whenever a port register is written; it knows nothing else about them.
pub trait ViaPortConnections {
    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64);

    /// Temporary: the IC32 latch value, passed to the JS device so it can
    /// cross-check its own copy. Goes when JS no longer keeps one.
    fn ic32_latch(&self) -> u8;
}

impl<T: ViaPortConnections> ViaPortConnections for Rc<RefCell<T>> {
    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64) {
        self.borrow_mut().update(port_a, port_b, cycles);
    }

    fn ic32_latch(&self) -> u8 {
        self.borrow().ic32_latch()
    }
}

pub struct ViaPortState {
    pub value: u8,
    pub output_mask: u8,
}

impl ViaPortState {
    pub fn resolve_floating(&self) -> u8 {
        // floating lines treated as high
        (self.value & self.output_mask) | !self.output_mask
    }
}
