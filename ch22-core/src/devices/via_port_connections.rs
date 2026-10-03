use std::{cell::RefCell, rc::Rc};

/// What is wired to a VIA's ports. The VIA reports the state of both ports
/// whenever a port register is written; it knows nothing else about them.
pub trait ViaPortConnections {
    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64);
}

impl<T: ViaPortConnections> ViaPortConnections for Rc<RefCell<T>> {
    fn update(&mut self, port_a: ViaPortState, port_b: ViaPortState, cycles: u64) {
        self.borrow_mut().update(port_a, port_b, cycles);
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
