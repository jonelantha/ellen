use crate::cycle_manager::{CycleManagerTrait, CycleOp};

use super::memory_access::MemoryAccess;

use AddressMode::*;

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

impl AddressMode {
    pub fn address<T: CycleManagerTrait>(&self, memory_access: &mut MemoryAccess<T>) -> u16 {
        match self {
            Immediate => panic!(),

            ZeroPage => memory_access.read_immediate() as u16,

            ZeroPageIndexed(index) => {
                let base_address = memory_access.read_immediate();

                memory_access.phantom_read(base_address as u16);

                base_address.wrapping_add(*index) as u16
            }

            Absolute => memory_access.read_immediate_16(),

            AbsoluteIndexed(_) => self.address_with_carry(memory_access).0,

            Indirect => {
                let base_address = memory_access.read_immediate_16();

                memory_access.read_16(base_address, CycleOp::None)
            }

            IndexedIndirect(index) => {
                let address = ZeroPageIndexed(*index).address(memory_access);

                memory_access.read_16(address, CycleOp::None)
            }

            IndirectIndexed(index) => {
                let zero_page_address = ZeroPage.address(memory_access);

                let base_address = memory_access.read_16(zero_page_address, CycleOp::None);

                let (address, carry_result) = address_offset_unsigned(base_address, *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    memory_access.phantom_read(intermediate);
                } else {
                    memory_access.phantom_read(address);
                }

                address
            }

            Relative => {
                let rel_address = memory_access.read_immediate() as i8;

                memory_access.phantom_program_counter_read();

                let (address, carry_result) =
                    address_offset_signed(*memory_access.program_counter, rel_address);

                if let CarryResult::Carried { intermediate } = carry_result {
                    memory_access.phantom_read(intermediate);
                }

                address
            }
        }
    }

    pub fn address_with_carry<T: CycleManagerTrait>(
        &self,
        memory_access: &mut MemoryAccess<T>,
    ) -> (u16, bool) {
        match self {
            AbsoluteIndexed(index) => {
                let (address, carry_result) =
                    address_offset_unsigned(memory_access.read_immediate_16(), *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    memory_access.phantom_read(intermediate);

                    (address, true)
                } else {
                    memory_access.phantom_read(address);

                    (address, false)
                }
            }

            _ => panic!(),
        }
    }

    pub fn data<T: CycleManagerTrait>(&self, memory_access: &mut MemoryAccess<T>) -> u8 {
        match self {
            Immediate => memory_access.read_immediate(),

            ZeroPage | ZeroPageIndexed(_) | Absolute | IndexedIndirect(_) | Indirect | Relative => {
                let address = self.address(memory_access);

                memory_access.read(address, CycleOp::CheckInterrupt)
            }

            AbsoluteIndexed(index) => {
                let (address, carry_result) =
                    address_offset_unsigned(memory_access.read_immediate_16(), *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    memory_access.phantom_read(intermediate);
                }

                memory_access.read(address, CycleOp::CheckInterrupt)
            }

            IndirectIndexed(index) => {
                let zero_page_address = ZeroPage.address(memory_access);

                let base_address = memory_access.read_16(zero_page_address, CycleOp::None);

                let (address, carry_result) = address_offset_unsigned(base_address, *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    memory_access.phantom_read(intermediate);
                }

                memory_access.read(address, CycleOp::CheckInterrupt)
            }
        }
    }
}

fn address_offset(base_address: u16, offset: i16) -> (u16, CarryResult) {
    let address = base_address.wrapping_add(offset as u16);

    let carried = address & 0xff00 != base_address & 0xff00;

    if carried {
        let intermediate = (base_address & 0xff00) | (address & 0x00ff);
        (address, CarryResult::Carried { intermediate })
    } else {
        (address, CarryResult::NoCarry)
    }
}

fn address_offset_unsigned(base_address: u16, offset: u8) -> (u16, CarryResult) {
    address_offset(base_address, offset as i16)
}

fn address_offset_signed(base_address: u16, offset: i8) -> (u16, CarryResult) {
    address_offset(base_address, offset as i16)
}

enum CarryResult {
    Carried { intermediate: u16 },
    NoCarry,
}
