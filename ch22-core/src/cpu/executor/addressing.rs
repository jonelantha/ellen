use crate::bus::*;
use crate::cpu::registers::*;
use crate::word::*;

use AddressMode::*;

pub struct ImmediateAccess<'a, B: Bus> {
    pub bus: &'a mut B,
    pub program_counter: &'a mut Word,
}

impl<'a, B: Bus> ImmediateAccess<'a, B> {
    fn immediate_fetch(&mut self) -> u8 {
        let value = self.bus.read(*self.program_counter, CycleOp::None);

        advance_program_counter(self.program_counter);

        value
    }

    fn immediate_fetch_word(&mut self) -> Word {
        Word {
            low: self.immediate_fetch(),
            high: self.immediate_fetch(),
        }
    }
}

pub struct AddressModeAccess<'a, B: Bus> {
    pub bus: &'a mut B,
    pub program_counter: &'a mut Word,
}

impl<'a, B: Bus> AddressModeAccess<'a, B> {
    pub fn address(&mut self, address_mode: &AddressMode) -> Word {
        let mut immediate_access = ImmediateAccess {
            bus: self.bus,
            program_counter: self.program_counter,
        };

        match address_mode {
            Immediate => panic!(),

            ZeroPage => Word::zero_page(immediate_access.immediate_fetch()),

            ZeroPageIndexed(index) => {
                let base_address = Word::zero_page(immediate_access.immediate_fetch());

                self.bus.phantom_read(base_address);

                base_address.same_page_add(*index)
            }

            Absolute => immediate_access.immediate_fetch_word(),

            AbsoluteIndexed(index) => {
                let base_address = immediate_access.immediate_fetch_word();

                let (address, offset_result) = base_address.paged_add(*index);

                match offset_result {
                    OffsetResult::CrossedPage(intermediate) => self.bus.phantom_read(intermediate),

                    OffsetResult::SamePage => self.bus.phantom_read(address),
                }

                address
            }

            Indirect => {
                let base_address = immediate_access.immediate_fetch_word();

                read_word(self.bus, base_address, CycleOp::None)
            }

            IndexedIndirect(index) => {
                let address = self.address(&ZeroPageIndexed(*index));

                read_word(self.bus, address, CycleOp::None)
            }

            IndirectIndexed(index) => {
                let zero_page_address = self.address(&ZeroPage);

                let base_address = read_word(self.bus, zero_page_address, CycleOp::None);

                let (address, offset_result) = base_address.paged_add(*index);

                match offset_result {
                    OffsetResult::CrossedPage(intermediate) => self.bus.phantom_read(intermediate),

                    OffsetResult::SamePage => self.bus.phantom_read(address),
                }

                address
            }

            Relative => {
                let rel_address = immediate_access.immediate_fetch();

                self.phantom_immediate_read();

                let (address, offset_result) = if rel_address & 0x80 != 0 {
                    self.program_counter.paged_subtract(rel_address)
                } else {
                    self.program_counter.paged_add(rel_address)
                };

                if let OffsetResult::CrossedPage(intermediate) = offset_result {
                    self.bus.phantom_read(intermediate);
                }

                address
            }
        }
    }

    pub fn address_with_carry(&mut self, address_mode: &AddressMode) -> (Word, bool) {
        let mut immediate_access = ImmediateAccess {
            bus: self.bus,
            program_counter: self.program_counter,
        };

        match address_mode {
            AbsoluteIndexed(index) => {
                let base_address = immediate_access.immediate_fetch_word();

                let (address, offset_result) = base_address.paged_add(*index);

                if let OffsetResult::CrossedPage(intermediate) = offset_result {
                    self.bus.phantom_read(intermediate);

                    (address, true)
                } else {
                    self.bus.phantom_read(address);

                    (address, false)
                }
            }

            _ => panic!(),
        }
    }

    pub fn data(&mut self, address_mode: &AddressMode) -> u8 {
        let mut immediate_access = ImmediateAccess {
            bus: self.bus,
            program_counter: self.program_counter,
        };

        match address_mode {
            Immediate => immediate_access.immediate_fetch(),

            ZeroPage | ZeroPageIndexed(_) | Absolute | IndexedIndirect(_) | Indirect | Relative => {
                let address = self.address(address_mode);

                self.bus.read(address, CycleOp::CheckInterrupt)
            }

            AbsoluteIndexed(index) => {
                let base_address = immediate_access.immediate_fetch_word();

                let (address, offset_result) = base_address.paged_add(*index);

                if let OffsetResult::CrossedPage(intermediate) = offset_result {
                    self.bus.phantom_read(intermediate);
                }

                self.bus.read(address, CycleOp::CheckInterrupt)
            }

            IndirectIndexed(index) => {
                let zero_page_address = self.address(&ZeroPage);

                let base_address = read_word(self.bus, zero_page_address, CycleOp::None);

                let (address, offset_result) = base_address.paged_add(*index);

                if let OffsetResult::CrossedPage(intermediate) = offset_result {
                    self.bus.phantom_read(intermediate);
                }

                self.bus.read(address, CycleOp::CheckInterrupt)
            }
        }
    }

    pub fn phantom_immediate_read(&mut self) {
        self.bus.phantom_read(*self.program_counter);
    }
}

pub fn read_immediate<B: Bus>(bus: &mut B, program_counter: &mut Word) -> u8 {
    let value = bus.read(*program_counter, CycleOp::None);

    advance_program_counter(program_counter);

    value
}

pub enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageIndexed(u8),
    Absolute,
    AbsoluteIndexed(u8),
    Indirect,
    IndexedIndirect(u8),
    IndirectIndexed(u8),
    Relative,
}

pub fn read_word<B: Bus>(bus: &mut B, address: Word, op: CycleOp) -> Word {
    Word {
        low: bus.read(address, op),
        high: bus.read(address.same_page_add(1), op),
    }
}
